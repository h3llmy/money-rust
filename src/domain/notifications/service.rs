use crate::domain::notifications::entity::NotificationInbox;
use crate::domain::notifications::NotificationRepository;
use crate::domain::transactions::service::TransactionService;
use crate::domain::pockets::PocketRepository;
use crate::domain::categories::CategoryRepository;
use crate::infrastructure::ai::AiClient;
use crate::shared::pagination::PaginationQuery;
use uuid::Uuid;
use std::sync::Arc;


pub struct NotificationService {
    repo: Arc<dyn NotificationRepository>,
    pocket_repo: Arc<dyn PocketRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    transaction_service: Arc<TransactionService>,
    ai_client: Arc<dyn AiClient>,
}

impl NotificationService {
    pub fn new(
        repo: Arc<dyn NotificationRepository>,
        pocket_repo: Arc<dyn PocketRepository>,
        category_repo: Arc<dyn CategoryRepository>,
        transaction_service: Arc<TransactionService>,
        ai_client: Arc<dyn AiClient>,
    ) -> Self {
        Self { 
            repo, 
            pocket_repo, 
            category_repo, 
            transaction_service, 
            ai_client 
        }
    }

    pub async fn list_unresolved(&self, user_id: Uuid, pagination: PaginationQuery) -> Result<(Vec<NotificationInbox>, u64), String> {
        self.repo.find_unresolved(user_id, pagination).await
    }

    pub async fn ingest_sync(&self, user_id: Uuid, notifications: Vec<NotificationInbox>) -> Result<(), String> {
        let full_page = PaginationQuery { limit: Some(1000), ..Default::default() };
        let (pockets, _) = self.pocket_repo.find_all(user_id, full_page.clone()).await?;
        let (categories, _) = self.category_repo.find_all(user_id, full_page).await?;

        for mut n in notifications {
            let title = n.raw_title.as_deref().unwrap_or("");
            let body = &n.raw_body;

            let mut pocket_id = None;
            let mut category_id = None;
            let mut destination_pocket_id = None;
            let mut is_ignored = false;

            match self.ai_client.parse_notification(title, body).await {
                Ok(Some(ai_data)) => {
                    if !ai_data.is_transaction {
                        is_ignored = true;
                    } else {
                        pocket_id = if let Some(ref pocket_name) = ai_data.pocket {
                            pockets.iter()
                                .find(|p| p.name.to_lowercase().contains(&pocket_name.to_lowercase()) || pocket_name.to_lowercase().contains(&p.name.to_lowercase()))
                                .map(|p| p.id)
                        } else {
                            None
                        };

                        category_id = if let Some(ref cat_name) = ai_data.category {
                            categories.iter()
                                .find(|c| c.name.to_lowercase().contains(&cat_name.to_lowercase()) || cat_name.to_lowercase().contains(&c.name.to_lowercase()))
                                .map(|c| c.id)
                        } else {
                            None
                        };

                        destination_pocket_id = if let Some(ref dest_name) = ai_data.destination_pocket {
                            pockets.iter()
                                .find(|p| p.name.to_lowercase().contains(&dest_name.to_lowercase()) || dest_name.to_lowercase().contains(&p.name.to_lowercase()))
                                .map(|p| p.id)
                        } else {
                            None
                        };

                        n.amount = ai_data.amount;
                        n.type_ = ai_data.transaction_type;
                        n.pocket_id = pocket_id;
                        n.category_id = category_id;
                        n.destination_pocket_id = destination_pocket_id;
                        n.title = ai_data.title;
                    }
                },
                Ok(None) => {
                    is_ignored = true;
                },
                Err(e) => {
                    tracing::error!("AI notification parsing failed: {}", e);
                }
            }

            if is_ignored {
                tracing::info!("AI determined that the notification is not a financial transaction. Marking as ignored.");
                n.status = crate::domain::notifications::entity::NotificationStatus::Ignored;
                self.repo.save(n).await?;
                continue;
            }

            // Fallback parsing for fields that remain None (or if AI failed entirely)
            if n.amount.is_none() {
                n.amount = extract_amount_fallback(title, body);
            }

            let has_amount = match &n.amount {
                Some(amt) => amt > &bigdecimal::BigDecimal::from(0),
                None => false,
            };

            if !has_amount {
                tracing::info!("Notification does not contain a valid transaction amount. Marking as ignored.");
                n.status = crate::domain::notifications::entity::NotificationStatus::Ignored;
                self.repo.save(n).await?;
                continue;
            }

            if n.type_.is_none() {
                n.type_ = Some(extract_type_fallback(title, body));
            }
            if n.pocket_id.is_none() {
                n.pocket_id = pockets.iter()
                     .find(|p| format!("{} {}", title, body).to_lowercase().contains(&p.name.to_lowercase()))
                     .map(|p| p.id);
                pocket_id = n.pocket_id;
            }
            if n.title.is_none() {
                n.title = n.raw_title.clone().or_else(|| Some("Notification Transaction".to_string()));
            }

            // Always create a pending transaction, even if AI is offline, fails, or returns None!
            if !pockets.is_empty() {
                let final_pocket_id = pocket_id.unwrap_or_else(|| pockets.first().map(|p| p.id).unwrap_or_default());
                let final_amount = n.amount.clone().unwrap_or_else(|| bigdecimal::BigDecimal::from(0));
                let final_type = n.type_.clone().unwrap_or_else(|| "expense".to_string());
                let final_title = n.title.clone()
                    .or_else(|| n.raw_title.clone())
                    .unwrap_or_else(|| "Notification Transaction".to_string());

                let tx = self.transaction_service.create_transaction(
                    user_id,
                    final_pocket_id,
                    category_id,
                    final_amount,
                    final_type,
                    final_title,
                    n.received_at,
                    destination_pocket_id,
                    Some(n.raw_body.clone()),
                    "pending".to_string(),
                ).await?;

                n.transaction_id = Some(tx.id);
                n.status = crate::domain::notifications::entity::NotificationStatus::Processed;
            } else {
                n.status = crate::domain::notifications::entity::NotificationStatus::Pending;
            }

            self.repo.save(n).await?;
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn process_notification(&self, user_id: Uuid, id: Uuid) -> Result<(), String> {
        let notification = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Notification not found".to_string())?;

        if notification.user_id != user_id {
            return Err("Unauthorized notification access".to_string());
        }

        tracing::debug!("Processing notification {} with AI", id);
        
        let title = notification.raw_title.as_deref().unwrap_or("");
        let body = &notification.raw_body;

        match self.ai_client.parse_notification(title, body).await {
            Ok(Some(ai_data)) => {
                if !ai_data.is_transaction {
                    tracing::info!("Notification {} is determined not to be a transaction by AI. Marking as ignored.", id);
                    let mut n = notification;
                    n.status = crate::domain::notifications::entity::NotificationStatus::Ignored;
                    self.repo.save(n).await?;
                    return Ok(());
                }

                let full_page = PaginationQuery { limit: Some(100), ..Default::default() };
                let (pockets, _) = self.pocket_repo.find_all(user_id, full_page.clone()).await?;
                let (categories, _) = self.category_repo.find_all(user_id, full_page).await?;

                if pockets.is_empty() {
                    tracing::warn!("No pockets found. Skipping transaction creation for notification {}", id);
                    return Ok(());
                }

                let pocket_id = if let Some(ref pocket_name) = ai_data.pocket {
                    pockets.iter()
                        .find(|p| p.name.to_lowercase().contains(&pocket_name.to_lowercase()) || pocket_name.to_lowercase().contains(&p.name.to_lowercase()))
                        .map(|p| p.id)
                        .unwrap_or_else(|| pockets.first().map(|p| p.id).unwrap_or_default())
                } else {
                    pockets.first().map(|p| p.id).unwrap_or_default()
                };

                let category_id = if let Some(ref cat_name) = ai_data.category {
                    categories.iter()
                        .find(|c| c.name.to_lowercase().contains(&cat_name.to_lowercase()) || cat_name.to_lowercase().contains(&c.name.to_lowercase()))
                        .map(|c| c.id)
                } else {
                    None
                };

                let destination_pocket_id = if let Some(ref dest_name) = ai_data.destination_pocket {
                    pockets.iter()
                        .find(|p| p.name.to_lowercase().contains(&dest_name.to_lowercase()) || dest_name.to_lowercase().contains(&p.name.to_lowercase()))
                        .map(|p| p.id)
                } else {
                    None
                };

                let amount = ai_data.amount.unwrap();
                let type_ = ai_data.transaction_type.unwrap();
                let title = ai_data.title.unwrap();
                
                let time_window_start = notification.received_at - chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));
                let time_window_end = notification.received_at + chrono::Duration::try_days(1).unwrap_or_else(|| chrono::Duration::days(1));
                
                let (existing_txs, _) = self.transaction_service.list_transactions(
                    user_id,
                    Some(pocket_id),
                    Some(time_window_start),
                    Some(time_window_end),
                    None,
                    PaginationQuery { limit: Some(50), ..Default::default() }
                ).await?;

                let is_duplicate = existing_txs.into_iter().find(|tx| {
                    if tx.amount == amount {
                        if type_ == "income" && tx.type_ == "transfer" && tx.destination_pocket_id == Some(pocket_id) {
                            return true;
                        }
                        if type_ == "expense" && tx.type_ == "transfer" && tx.pocket_id == pocket_id {
                            return true;
                        }
                        if tx.type_ == type_ && tx.pocket_id == pocket_id {
                            return true;
                        }
                    }
                    false
                });

                if let Some(duplicate_tx) = is_duplicate {
                    tracing::info!("Duplicate transaction detected for notification {}. Linking to existing transaction {}", id, duplicate_tx.id);
                    self.repo.update_status(id, "processed".to_string(), Some(duplicate_tx.id)).await?;
                    return Ok(());
                }
                
                let tx = self.transaction_service.create_transaction(
                    user_id,
                    pocket_id,
                    category_id,
                    amount,
                    type_,
                    title,
                    notification.received_at,
                    destination_pocket_id,
                    Some(notification.raw_body.clone()),
                    "resolved".to_string(),
                ).await?;

                self.repo.update_status(id, "processed".to_string(), Some(tx.id)).await?;
                Ok(())
            },
            Ok(None) => {
                tracing::info!("Notification {} ignored (not a transaction).", id);
                self.repo.update_status(id, "ignored".to_string(), None).await?;
                Ok(())
            },
            Err(e) => {
                tracing::error!("AI parsing failed for notification {}: {}", id, e);
                self.repo.update_status(id, "failed".to_string(), None).await?;
                Err(e)
            }
        }
    }
}

fn extract_amount_fallback(title: &str, body: &str) -> Option<bigdecimal::BigDecimal> {
    use regex::Regex;
    use std::str::FromStr;
    
    let text = format!("{} {}", title, body);
    
    // 1. Look for Rp or RP followed by optional spaces and digits/dots
    if let Ok(re) = Regex::new(r"(?i)rp\s*([0-9\.\,]+)") {
        if let Some(cap) = re.captures(&text) {
            if let Some(val_match) = cap.get(1) {
                let val_str = val_match.as_str();
                let cleaned = val_str.replace(".", "").replace(",", ".");
                if let Ok(amount) = bigdecimal::BigDecimal::from_str(&cleaned) {
                    return Some(amount);
                }
            }
        }
    }
    
    // 2. Look for numbers like 80.000
    if let Ok(re_num) = Regex::new(r"\b([0-9]{1,3}(\.[0-9]{3})+)\b") {
        if let Some(cap) = re_num.captures(&text) {
            if let Some(val_match) = cap.get(1) {
                let val_str = val_match.as_str();
                let cleaned = val_str.replace(".", "");
                if let Ok(amount) = bigdecimal::BigDecimal::from_str(&cleaned) {
                    return Some(amount);
                }
            }
        }
    }
    
    None
}

fn extract_type_fallback(title: &str, body: &str) -> String {
    let text = format!("{} {}", title, body).to_lowercase();
    if text.contains("uang masuk") || text.contains("menerima") || text.contains("received") || text.contains("masuk") || text.contains("tambah saldo") {
        "income".to_string()
    } else if text.contains("transfer") || text.contains("kirim") || text.contains("pindah") {
        "transfer".to_string()
    } else {
        "expense".to_string()
    }
}
