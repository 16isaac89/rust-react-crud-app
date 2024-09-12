use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[derive(Default)]
pub struct FilterOptions{
    pub page:Option<usize>,
    pub limit:Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ParamOptions{
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNoteSchema{
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNoteSchema{
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub published: Option<bool>,
}

pub struct SaveUser{
    pub id: String,
    pub first_name:String,
    pub second_name:String,
    pub email:String,
    pub active:Option<String>,
    pub password:String,
    pub dob: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
