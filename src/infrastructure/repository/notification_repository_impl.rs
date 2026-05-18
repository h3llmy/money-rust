use crate::domain::notifications::entity::{NotificationInbox, NotificationStatus};
use crate::domain::notifications::NotificationRepository;
use crate::shared::pagination::PaginationQuery;
use async_trait::async_trait;
use std::sync::RwLock;
use uuid::Uuid;

pub struct InMemoryNotificationRepository {
    notifications: RwLock<Vec<NotificationInbox>>,
}

impl InMemoryNotificationRepository {
    pub fn new() -> Self {
        Self {
            notifications: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl NotificationRepository for InMemoryNotificationRepository {
    async fn find_unresolved(&self, pagination: PaginationQuery) -> Result<(Vec<NotificationInbox>, u64), String> {
        let n_list = self.notifications.read().map_err(|e| e.to_string())?;
        
        let filtered: Vec<NotificationInbox> = n_list.iter()
            .filter(|n| {
                let is_processed = matches!(n.status, NotificationStatus::Processed);
                let is_ignored = matches!(n.status, NotificationStatus::Ignored);
                !is_processed && !is_ignored
            })
            .filter(|n| {
                if let Some(ref search) = pagination.get_search() {
                    n.raw_body.to_lowercase().contains(&search.to_lowercase())
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        let total = filtered.len() as u64;
        
        // Apply sorting (by received_at desc) and pagination (limit, offset)
        let mut sorted = filtered;
        sorted.sort_by(|a, b| b.received_at.cmp(&a.received_at));
        
        let offset = pagination.get_offset() as usize;
        let limit = pagination.get_limit() as usize;
        
        let paginated = sorted.into_iter()
            .skip(offset)
            .take(limit)
            .collect();
            
        Ok((paginated, total))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<NotificationInbox>, String> {
        let n_list = self.notifications.read().map_err(|e| e.to_string())?;
        Ok(n_list.iter().find(|n| n.id == id).cloned())
    }

    async fn save(&self, notification: NotificationInbox) -> Result<NotificationInbox, String> {
        let mut n_list = self.notifications.write().map_err(|e| e.to_string())?;
        if let Some(pos) = n_list.iter().position(|n| n.id == notification.id) {
            n_list[pos] = notification.clone();
        } else {
            n_list.push(notification.clone());
        }
        Ok(notification)
    }

    async fn update_status(&self, id: Uuid, status: String, transaction_id: Option<Uuid>) -> Result<(), String> {
        let mut n_list = self.notifications.write().map_err(|e| e.to_string())?;
        if let Some(n) = n_list.iter_mut().find(|n| n.id == id) {
            n.status = NotificationStatus::from(status);
            n.transaction_id = transaction_id;
            Ok(())
        } else {
            Err("Notification not found".to_string())
        }
    }
}
