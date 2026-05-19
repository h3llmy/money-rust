use crate::domain::transactions::entity::Transaction;
use crate::domain::transactions::TransactionRepository;
use crate::infrastructure::database::DbPool;
use crate::infrastructure::repository::TransactionDb;
use crate::infrastructure::schema::transactions;
use crate::shared::pagination::{PaginationQuery, SortOrder};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub struct DieselTransactionRepository {
    pool: DbPool,
}

impl DieselTransactionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for DieselTransactionRepository {
    async fn find_all(
        &self, 
        user_id: Uuid,
        pocket_id: Option<Uuid>, 
        start_date: Option<DateTime<Utc>>, 
        end_date: Option<DateTime<Utc>>, 
        type_: Option<String>,
        pagination: PaginationQuery,
    ) -> Result<(Vec<Transaction>, u64), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        // Helper to apply filters to a boxed query, wrapped in a block to be an expression
        macro_rules! apply_filters {
            ($query:expr) => {{
                let mut q = $query;
                q = q.filter(transactions::user_id.eq(user_id));
                if let Some(pid) = pocket_id {
                    q = q.filter(transactions::pocket_id.eq(pid).or(transactions::destination_pocket_id.eq(pid)));
                }
                if let Some(sd) = start_date {
                    q = q.filter(transactions::transaction_time.ge(sd));
                }
                if let Some(ed) = end_date {
                    q = q.filter(transactions::transaction_time.le(ed));
                }
                if let Some(ref t) = type_ {
                    q = q.filter(transactions::type_.eq(t));
                }
                if let Some(ref search) = pagination.get_search() {
                    q = q.filter(transactions::title.ilike(format!("%{}%", search)));
                }
                q
            }};
        }

        let total_data = apply_filters!(transactions::table.into_boxed())
            .count()
            .get_result::<i64>(&mut conn)
            .await
            .map_err(|e| e.to_string())? as u64;

        let mut list_query = apply_filters!(transactions::table.into_boxed());

        // Apply Sorting
        list_query = match (pagination.get_sort().as_deref(), pagination.get_sort_order()) {
            (Some("amount"), Some(SortOrder::Asc)) => list_query.order(transactions::amount.asc()),
            (Some("amount"), _) => list_query.order(transactions::amount.desc()),
            (_, Some(SortOrder::Asc)) => list_query.order(transactions::transaction_time.asc()),
            _ => list_query.order(transactions::transaction_time.desc()),
        };

        let results = list_query
            .limit(pagination.get_limit() as i64)
            .offset(pagination.get_offset())
            .load::<TransactionDb>(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        
        Ok((results.into_iter().map(map_db_to_domain).collect(), total_data))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Transaction>, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let result = transactions::table
            .filter(transactions::id.eq(id))
            .first::<TransactionDb>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;
        
        Ok(result.map(map_db_to_domain))
    }

    async fn save(&self, transaction: Transaction) -> Result<Transaction, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        let db_model = TransactionDb {
            id: transaction.id,
            user_id: transaction.user_id,
            pocket_id: transaction.pocket_id,
            category_id: transaction.category_id,
            amount: transaction.amount,
            type_: transaction.type_,
            title: transaction.title,
            transaction_time: transaction.transaction_time,
            destination_pocket_id: transaction.destination_pocket_id,
            description: transaction.description,
            status: transaction.status,
        };

        diesel::insert_into(transactions::table)
            .values(&db_model)
            .on_conflict(transactions::id)
            .do_update()
            .set(&db_model)
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(map_db_to_domain(db_model))
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        diesel::delete(transactions::table.filter(transactions::id.eq(id)))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn map_db_to_domain(db: TransactionDb) -> Transaction {
    Transaction {
        id: db.id,
        user_id: db.user_id,
        pocket_id: db.pocket_id,
        category_id: db.category_id,
        amount: db.amount,
        type_: db.type_,
        title: db.title,
        transaction_time: db.transaction_time,
        destination_pocket_id: db.destination_pocket_id,
        description: db.description,
        status: db.status,
    }
}
