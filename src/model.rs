use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct NoteModel {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub published: i8,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NoteModelResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub published: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

