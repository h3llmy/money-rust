use uuid::Uuid;

pub struct Category {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub type_: String, // income or expense
}
