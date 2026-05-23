use crate::domain::transactions::entity::Transaction;
use crate::domain::transactions::TransactionRepository;
use crate::domain::pockets::PocketRepository;
use crate::domain::categories::CategoryRepository;
use crate::infrastructure::ai::AiClient;
use crate::shared::pagination::PaginationQuery;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

pub struct TransactionService {
    repo: Arc<dyn TransactionRepository>,
    pocket_repo: Arc<dyn PocketRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    ai_client: Arc<dyn AiClient>,
}

impl TransactionService {
    pub fn new(
        repo: Arc<dyn TransactionRepository>,
        pocket_repo: Arc<dyn PocketRepository>,
        category_repo: Arc<dyn CategoryRepository>,
        ai_client: Arc<dyn AiClient>,
    ) -> Self {
        Self { repo, pocket_repo, category_repo, ai_client }
    }

    pub async fn list_transactions(
        &self,
        user_id: Uuid,
        pocket_id: Option<Uuid>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        type_: Option<String>,
        pagination: PaginationQuery,
    ) -> Result<(Vec<Transaction>, u64), String> {
        self.repo.find_all(user_id, pocket_id, start_date, end_date, type_, pagination).await
    }

    pub async fn get_transaction_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Transaction>, String> {
        let tx = self.repo.find_by_id(id).await?;
        if let Some(t) = tx {
            let pocket = self.pocket_repo.find_by_id(t.pocket_id).await?;
            if let Some(p) = pocket {
                if p.user_id == user_id {
                    return Ok(Some(t));
                }
            }
        }
        Ok(None)
    }

    pub async fn create_transaction(
        &self,
        user_id: Uuid,
        pocket_id: Uuid,
        category_id: Option<Uuid>,
        amount: BigDecimal,
        type_: String,
        title: String,
        transaction_time: DateTime<Utc>,
        destination_pocket_id: Option<Uuid>,
        description: Option<String>,
        status: String,
    ) -> Result<Transaction, String> {
        let pocket = self.pocket_repo.find_by_id(pocket_id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
        if pocket.user_id != user_id {
            return Err("Unauthorized pocket access".to_string());
        }

        if type_ == "transfer" {
            if let Some(dest_id) = destination_pocket_id {
                let dest_pocket = self.pocket_repo.find_by_id(dest_id).await?
                    .ok_or_else(|| "Destination pocket not found".to_string())?;
                if dest_pocket.user_id != user_id {
                    return Err("Unauthorized destination pocket access".to_string());
                }
            }
        }

        let transaction = Transaction {
            id: Uuid::new_v4(),
            user_id,
            pocket_id,
            category_id,
            amount: amount.clone(),
            type_: type_.clone(),
            title,
            transaction_time,
            destination_pocket_id,
            description,
            status: status.clone(),
        };

        let saved = self.repo.save(transaction).await?;

        if status != "rejected" {
            let balance_change = match type_.as_str() {
                "income" => amount.clone(),
                "expense" => -amount.clone(),
                "transfer" => -amount.clone(),
                _ => BigDecimal::from(0),
            };

            self.pocket_repo.update_balance(pocket_id, balance_change).await?;

            if type_ == "transfer" {
                if let Some(dest_id) = destination_pocket_id {
                    self.pocket_repo.update_balance(dest_id, amount).await?;
                }
            }
        }

        Ok(saved)
    }

    pub async fn void_transaction(&self, id: Uuid, user_id: Uuid) -> Result<(), String> {
        let transaction = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Transaction not found".to_string())?;

        let pocket = self.pocket_repo.find_by_id(transaction.pocket_id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
        if pocket.user_id != user_id {
            return Err("Unauthorized transaction access".to_string());
        }

        if transaction.status != "rejected" {
            let reverse_amount = match transaction.type_.as_str() {
                "income" => -transaction.amount.clone(),
                "expense" => transaction.amount.clone(),
                "transfer" => transaction.amount.clone(),
                _ => BigDecimal::from(0),
            };

            self.pocket_repo.update_balance(transaction.pocket_id, reverse_amount).await?;

            if transaction.type_ == "transfer" {
                if let Some(dest_id) = transaction.destination_pocket_id {
                    self.pocket_repo.update_balance(dest_id, -transaction.amount).await?;
                }
            }
        }

        self.repo.delete(id).await
    }

    pub async fn update_transaction(
        &self,
        id: Uuid,
        user_id: Uuid,
        pocket_id: Uuid,
        category_id: Option<Uuid>,
        amount: BigDecimal,
        type_: String,
        title: String,
        transaction_time: DateTime<Utc>,
        destination_pocket_id: Option<Uuid>,
        description: Option<String>,
        status: Option<String>,
    ) -> Result<Transaction, String> {
        let old_tx = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Transaction not found".to_string())?;

        let old_pocket = self.pocket_repo.find_by_id(old_tx.pocket_id).await?
            .ok_or_else(|| "Original pocket not found".to_string())?;
        if old_pocket.user_id != user_id {
            return Err("Unauthorized pocket access".to_string());
        }

        let new_pocket = self.pocket_repo.find_by_id(pocket_id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
        if new_pocket.user_id != user_id {
            return Err("Unauthorized pocket access".to_string());
        }

        if type_ == "transfer" {
            if let Some(dest_id) = destination_pocket_id {
                let dest_pocket = self.pocket_repo.find_by_id(dest_id).await?
                    .ok_or_else(|| "Destination pocket not found".to_string())?;
                if dest_pocket.user_id != user_id {
                    return Err("Unauthorized destination pocket access".to_string());
                }
            }
        }

        let target_status = status.unwrap_or_else(|| old_tx.status.clone());

        // 1. Revert the old transaction's pocket balance effects (if it wasn't rejected)
        if old_tx.status != "rejected" {
            let old_reverse_amount = match old_tx.type_.as_str() {
                "income" => -old_tx.amount.clone(),
                "expense" => old_tx.amount.clone(),
                "transfer" => old_tx.amount.clone(),
                _ => BigDecimal::from(0),
            };
            self.pocket_repo.update_balance(old_tx.pocket_id, old_reverse_amount).await?;
            if old_tx.type_ == "transfer" {
                if let Some(dest_id) = old_tx.destination_pocket_id {
                    self.pocket_repo.update_balance(dest_id, -old_tx.amount).await?;
                }
            }
        }

        // 2. Apply the new transaction's pocket balance effects (if new status is not rejected)
        if target_status != "rejected" {
            let new_balance_change = match type_.as_str() {
                "income" => amount.clone(),
                "expense" => -amount.clone(),
                "transfer" => -amount.clone(),
                _ => BigDecimal::from(0),
            };
            self.pocket_repo.update_balance(pocket_id, new_balance_change).await?;
            if type_ == "transfer" {
                if let Some(dest_id) = destination_pocket_id {
                    self.pocket_repo.update_balance(dest_id, amount.clone()).await?;
                }
            }
        }

        // 3. Save the updated transaction
        let updated_tx = Transaction {
            id,
            user_id,
            pocket_id,
            category_id,
            amount,
            type_,
            title,
            transaction_time,
            destination_pocket_id,
            description,
            status: target_status,
        };

        self.repo.save(updated_tx).await
    }

    pub async fn resolve_transaction(
        &self,
        id: Uuid,
        user_id: Uuid,
        pocket_id: Option<Uuid>,
        category_id: Option<Uuid>,
        amount: Option<BigDecimal>,
        type_: Option<String>,
        title: Option<String>,
        destination_pocket_id: Option<Uuid>,
        description: Option<String>,
    ) -> Result<Transaction, String> {
        let mut tx = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Transaction not found".to_string())?;

        let pocket = self.pocket_repo.find_by_id(tx.pocket_id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
        if pocket.user_id != user_id {
            return Err("Unauthorized transaction access".to_string());
        }

        if tx.status != "pending" {
            return Err("Transaction is not pending".to_string());
        }

        if let Some(pid) = pocket_id {
            let new_pocket = self.pocket_repo.find_by_id(pid).await?
                .ok_or_else(|| "Pocket not found".to_string())?;
            if new_pocket.user_id != user_id {
                return Err("Unauthorized pocket access".to_string());
            }
        }

        if let Some(dest_id) = destination_pocket_id {
            let dest_pocket = self.pocket_repo.find_by_id(dest_id).await?
                .ok_or_else(|| "Destination pocket not found".to_string())?;
            if dest_pocket.user_id != user_id {
                return Err("Unauthorized destination pocket access".to_string());
            }
        }

        // 1. Revert the old (pending) transaction's pocket balance effects
        let old_reverse_amount = match tx.type_.as_str() {
            "income" => -tx.amount.clone(),
            "expense" => tx.amount.clone(),
            "transfer" => tx.amount.clone(),
            _ => BigDecimal::from(0),
        };
        self.pocket_repo.update_balance(tx.pocket_id, old_reverse_amount).await?;
        if tx.type_ == "transfer" {
            if let Some(dest_id) = tx.destination_pocket_id {
                self.pocket_repo.update_balance(dest_id, -tx.amount.clone()).await?;
            }
        }

        // Apply corrections
        if let Some(pid) = pocket_id {
            tx.pocket_id = pid;
        }
        if category_id.is_some() {
            tx.category_id = category_id;
        }
        if let Some(amt) = amount {
            tx.amount = amt;
        }
        if let Some(ty) = type_ {
            tx.type_ = ty;
        }
        if let Some(t) = title {
            tx.title = t;
        }
        if destination_pocket_id.is_some() {
            tx.destination_pocket_id = destination_pocket_id;
        }
        if description.is_some() {
            tx.description = description;
        }

        // 2. Apply the new corrected transaction's pocket balance effects
        let balance_change = match tx.type_.as_str() {
            "income" => tx.amount.clone(),
            "expense" => -tx.amount.clone(),
            "transfer" => -tx.amount.clone(),
            _ => BigDecimal::from(0),
        };

        self.pocket_repo.update_balance(tx.pocket_id, balance_change).await?;

        if tx.type_ == "transfer" {
            if let Some(dest_id) = tx.destination_pocket_id {
                self.pocket_repo.update_balance(dest_id, tx.amount.clone()).await?;
            }
        }

        tx.status = "resolved".to_string();
        self.repo.save(tx).await
    }

    pub async fn reject_transaction(&self, id: Uuid, user_id: Uuid) -> Result<Transaction, String> {
        let mut tx = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Transaction not found".to_string())?;

        let pocket = self.pocket_repo.find_by_id(tx.pocket_id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
        if pocket.user_id != user_id {
            return Err("Unauthorized transaction access".to_string());
        }

        if tx.status != "pending" {
            return Err("Transaction is not pending".to_string());
        }

        // Revert the balance changes of the pending transaction
        let reverse_amount = match tx.type_.as_str() {
            "income" => -tx.amount.clone(),
            "expense" => tx.amount.clone(),
            "transfer" => tx.amount.clone(),
            _ => BigDecimal::from(0),
        };

        self.pocket_repo.update_balance(tx.pocket_id, reverse_amount).await?;

        if tx.type_ == "transfer" {
            if let Some(dest_id) = tx.destination_pocket_id {
                self.pocket_repo.update_balance(dest_id, -tx.amount.clone()).await?;
            }
        }

        tx.status = "rejected".to_string();
        self.repo.save(tx).await
    }

    pub async fn ai_analyze(
        &self,
        user_id: Uuid,
        user_query: Option<&str>,
    ) -> Result<futures_util::stream::BoxStream<'static, Result<String, String>>, String> {
        // 1. Get query parameters from AI if query is provided
        let (pocket_id, category_id, start_date, end_date, type_, limit) = if let Some(q) = user_query {
            let current_date = Utc::now().to_rfc3339();
            let ai_query = self.ai_client.parse_transaction_query(q, &current_date).await.unwrap_or_default();
            
            let mut p_id = None;
            if let Some(p_name) = ai_query.pocket_name {
                let (pockets, _) = self.pocket_repo.find_all(user_id, PaginationQuery { limit: Some(100), ..Default::default() }).await?;
                p_id = pockets.iter().find(|p| p.name.to_lowercase().contains(&p_name.to_lowercase())).map(|p| p.id);
            }
            
            let mut c_id = None;
            if let Some(c_name) = ai_query.category_name {
                let (cats, _) = self.category_repo.find_all(user_id, PaginationQuery { limit: Some(100), ..Default::default() }).await?;
                c_id = cats.iter().find(|c| c.name.to_lowercase().contains(&c_name.to_lowercase())).map(|c| c.id);
            }
            
            let start = ai_query.start_date.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
            let end = ai_query.end_date.and_then(|d| DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
            
            (p_id, c_id, start, end, ai_query.transaction_type, ai_query.limit)
        } else {
            (None, None, None, None, None, None)
        };

        // 2. Fetch transactions based on filter
        let pagination = PaginationQuery {
            page: Some(1),
            limit: Some(limit.unwrap_or(200)),
            search: None,
            sort: None,
            sort_order: None,
        };

        let (mut data, _) = self.list_transactions(user_id, pocket_id, start_date, end_date, type_, pagination).await?;
        
        // Post-filter by category_id if needed
        if let Some(cid) = category_id {
            data.retain(|t| t.category_id == Some(cid));
        }

        // Fetch pockets and categories to populate names
        let (pockets_list, _) = self.pocket_repo.find_all(user_id, PaginationQuery { limit: Some(1000), ..Default::default() }).await?;
        let (categories_list, _) = self.category_repo.find_all(user_id, PaginationQuery { limit: Some(1000), ..Default::default() }).await?;
        
        let pockets_map: std::collections::HashMap<Uuid, crate::domain::pockets::dto::PocketResponse> = pockets_list
            .into_iter()
            .map(|p| (p.id, crate::domain::pockets::dto::PocketResponse::from(p)))
            .collect();
            
        let categories_map: std::collections::HashMap<Uuid, crate::domain::categories::dto::CategoryResponse> = categories_list
            .into_iter()
            .map(|c| (c.id, crate::domain::categories::dto::CategoryResponse::from(c)))
            .collect();

        let mut populated_data = Vec::new();
        for t in data {
            let pocket = pockets_map.get(&t.pocket_id).cloned();
            let category = t.category_id.and_then(|cid| categories_map.get(&cid).cloned());
            let destination_pocket = t.destination_pocket_id.and_then(|dpid| pockets_map.get(&dpid).cloned());

            populated_data.push(crate::domain::transactions::dto::TransactionResponse {
                id: t.id,
                pocket_id: t.pocket_id,
                pocket,
                category_id: t.category_id,
                category,
                amount: t.amount.to_string(),
                type_: t.type_,
                title: t.title,
                transaction_time: t.transaction_time,
                destination_pocket_id: t.destination_pocket_id,
                destination_pocket,
                description: t.description,
                status: t.status,
            });
        }

        let transactions_json = serde_json::to_string_pretty(&populated_data)
            .map_err(|e| e.to_string())?;

        // 3. Call AI to analyze
        self.ai_client.analyze_transactions(&transactions_json, user_query).await
    }
}
