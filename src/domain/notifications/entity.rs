use uuid::Uuid;
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

#[derive(Clone, Debug)]
pub struct NotificationInbox {
    pub id: Uuid,
    pub user_id: Uuid,
    pub app_package: String,
    pub raw_title: Option<String>,
    pub raw_body: String,
    pub received_at: DateTime<Utc>,
    pub status: NotificationStatus,
    pub transaction_id: Option<Uuid>,
    pub amount: Option<BigDecimal>,
    pub type_: Option<String>,
    pub pocket_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub destination_pocket_id: Option<Uuid>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum NotificationStatus {
    Pending,
    Processed,
    Failed,
    Ignored,
}

impl ToString for NotificationStatus {
    fn to_string(&self) -> String {
        match self {
            NotificationStatus::Pending => "pending".to_string(),
            NotificationStatus::Processed => "processed".to_string(),
            NotificationStatus::Failed => "failed".to_string(),
            NotificationStatus::Ignored => "ignored".to_string(),
        }
    }
}

impl From<String> for NotificationStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "processed" => NotificationStatus::Processed,
            "failed" => NotificationStatus::Failed,
            "ignored" => NotificationStatus::Ignored,
            _ => NotificationStatus::Pending,
        }
    }
}
