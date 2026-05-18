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

    pub async fn list_all_pockets(&self, pagination: PaginationQuery) -> Result<(Vec<Pocket>, u64), String> {
        self.repo.find_all(pagination).await
    }

    pub async fn get_pocket_by_id(&self, id: Uuid) -> Result<Option<Pocket>, String> {
        self.repo.find_by_id(id).await
    }

    pub async fn create_pocket(&self, name: String, pocket_type: String, currency: String, balance: BigDecimal) -> Result<Pocket, String> {
        let pocket = Pocket {
            id: Uuid::new_v4(),
            _user_id: Uuid::new_v4(), // Mocking user for now
            name,
            _pocket_type: pocket_type,
            currency,
            balance,
            _updated_at: chrono::Utc::now(),
        };
        self.repo.save(pocket).await
    }

    pub async fn update_pocket(&self, id: Uuid, name: String, pocket_type: String) -> Result<Pocket, String> {
        let mut pocket = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Pocket not found".to_string())?;
            
        pocket.name = name;
        pocket._pocket_type = pocket_type;
        
        self.repo.save(pocket).await
    }

    pub async fn delete_pocket(&self, id: Uuid) -> Result<(), String> {
        self.repo.delete(id).await
    }
}
