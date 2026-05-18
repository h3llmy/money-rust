use crate::domain::pockets::service::PocketService;
use crate::domain::notifications::service::NotificationService;
use crate::domain::categories::service::CategoryService;
use crate::domain::transactions::service::TransactionService;
use std::sync::Arc;

pub struct AppState {
    pub pocket_service: Arc<PocketService>,
    pub notification_service: Arc<NotificationService>,
    pub category_service: Arc<CategoryService>,
    pub transaction_service: Arc<TransactionService>,
}
