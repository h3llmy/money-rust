use crate::domain::transactions::entity::Transaction;
use crate::domain::transactions::TransactionRepository;
use crate::domain::pockets::PocketRepository;
use crate::shared::pagination::PaginationQuery;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

pub struct TransactionService {
    repo: Arc<dyn TransactionRepository>,
    pocket_repo: Arc<dyn PocketRepository>,
}

impl TransactionService {
    pub fn new(repo: Arc<dyn TransactionRepository>, pocket_repo: Arc<dyn PocketRepository>) -> Self {
        Self { repo, pocket_repo }
    }

    pub async fn list_transactions(
        &self,
        pocket_id: Option<Uuid>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        type_: Option<String>,
        pagination: PaginationQuery,
    ) -> Result<(Vec<Transaction>, u64), String> {
        self.repo.find_all(pocket_id, start_date, end_date, type_, pagination).await
    }

    pub async fn get_transaction_by_id(&self, id: Uuid) -> Result<Option<Transaction>, String> {
        self.repo.find_by_id(id).await
    }

    pub async fn create_transaction(
        &self,
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
        let transaction = Transaction {
            id: Uuid::new_v4(),
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

    pub async fn void_transaction(&self, id: Uuid) -> Result<(), String> {
        let transaction = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Transaction not found".to_string())?;

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

        if tx.status != "pending" {
            return Err("Transaction is not pending".to_string());
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

    pub async fn reject_transaction(&self, id: Uuid) -> Result<Transaction, String> {
        let mut tx = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Transaction not found".to_string())?;

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
}
