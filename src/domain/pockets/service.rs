use crate::domain::pockets::entity::Pocket;
use crate::domain::pockets::PocketRepository;
use crate::shared::pagination::PaginationQuery;
use std::sync::Arc;
use uuid::Uuid;
use bigdecimal::BigDecimal;

pub struct PocketService {
    repo: Arc<dyn PocketRepository>,
}

impl PocketService {
    pub fn new(repo: Arc<dyn PocketRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_all_pockets(&self, user_id: Uuid, pagination: PaginationQuery) -> Result<(Vec<Pocket>, u64), String> {
        self.repo.find_all(user_id, pagination).await
    }

    pub async fn get_pocket_by_id(&self, id: Uuid, user_id: Uuid) -> Result<Option<Pocket>, String> {
        let pocket = self.repo.find_by_id(id).await?;
        match pocket {
            Some(p) if p.user_id == user_id => Ok(Some(p)),
            _ => Ok(None),
        }
    }

    pub async fn create_pocket(&self, user_id: Uuid, name: String, pocket_type: String, currency: String, balance: BigDecimal) -> Result<Pocket, String> {
        let pocket = Pocket {
            id: Uuid::new_v4(),
            user_id,
            name,
            _pocket_type: pocket_type,
            currency,
            balance,
            _updated_at: chrono::Utc::now(),
        };
        self.repo.save(pocket).await
    }

    pub async fn update_pocket(&self, id: Uuid, user_id: Uuid, name: String, pocket_type: String) -> Result<Pocket, String> {
        let mut pocket = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
            
        if pocket.user_id != user_id {
            return Err("Unauthorized pocket access".to_string());
        }
            
        pocket.name = name;
        pocket._pocket_type = pocket_type;
        
        self.repo.save(pocket).await
    }

    pub async fn delete_pocket(&self, id: Uuid, user_id: Uuid) -> Result<(), String> {
        let pocket = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
            
        if pocket.user_id != user_id {
            return Err("Unauthorized pocket access".to_string());
        }

        self.repo.delete(id).await
    }
}
