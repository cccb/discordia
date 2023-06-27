use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Insert {
    pub id: u32,
}

#[derive(Debug, Clone, FromRow)]
pub struct InsertString {
    pub id: String,
}
