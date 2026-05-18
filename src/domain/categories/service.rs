use crate::domain::categories::entity::Category;
use crate::domain::categories::CategoryRepository;
use crate::shared::pagination::PaginationQuery;
use std::sync::Arc;
use uuid::Uuid;

pub struct CategoryService {
    repo: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_categories(&self, pagination: PaginationQuery) -> Result<(Vec<Category>, u64), String> {
        self.repo.find_all(pagination).await
    }

    pub async fn get_category_by_id(&self, id: Uuid) -> Result<Option<Category>, String> {
        self.repo.find_by_id(id).await
    }

    pub async fn create_category(&self, name: String, type_: String) -> Result<Category, String> {
        let category = Category {
            id: Uuid::new_v4(),
            user_id: None, // Simplified for now
            name,
            type_,
        };
        self.repo.save(category).await
    }

    pub async fn update_category(&self, id: Uuid, name: String, type_: String) -> Result<Category, String> {
        let mut category = self.repo.find_by_id(id).await?
            .ok_or_else(|| "Category not found".to_string())?;
        category.name = name;
        category.type_ = type_;
        self.repo.save(category).await
    }

    pub async fn delete_category(&self, id: Uuid) -> Result<(), String> {
        self.repo.delete(id).await
    }
}
