use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::domain::notifications::entity::NotificationInbox;
use utoipa::ToSchema;

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateNotificationRequest {
    pub app_package: String,
    pub raw_title: Option<String>,
    pub raw_body: String,
}

#[derive(Serialize, ToSchema)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub app_package: String,
    pub raw_title: Option<String>,
    pub raw_body: String,
    pub status: String,
    pub received_at: DateTime<Utc>,
    pub amount: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub pocket_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub destination_pocket_id: Option<Uuid>,
    pub title: Option<String>,
}

impl From<NotificationInbox> for NotificationResponse {
    fn from(n: NotificationInbox) -> Self {
        Self {
            id: n.id,
            app_package: n.app_package,
            raw_title: n.raw_title,
            raw_body: n.raw_body,
            status: n.status.to_string(),
            received_at: n.received_at,
            amount: n.amount.map(|a| a.to_string()),
            type_: n.type_,
            pocket_id: n.pocket_id,
            category_id: n.category_id,
            destination_pocket_id: n.destination_pocket_id,
            title: n.title,
        }
    }
}
