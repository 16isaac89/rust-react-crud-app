use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::{NoteModel, NoteModelResponse},
    schema::{CreateNoteSchema, FilterOptions, ParamOptions, UpdateNoteSchema},
    AppState,
};

fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        title: note.title.to_owned(),
        content: note.content.to_owned(),
        category: note.category.to_owned(),
        published: note.published != 0,
        created_at: note.created_at.unwrap(),
        updated_at: note.updated_at.unwrap(),
    }
}

pub async fn note_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let notes = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes ORDER by id LIMIT ? OFFSET ?"#,
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status":"fail",
            "message":format!("Database error: {}",e)
        });

        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;
    let note_responses = notes
        .iter()
        .map(|note| filter_db_record(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response = serde_json::json!({
        "status":"success",
        "results":note_responses.len(),
        "notes":note_responses
    });
    Ok(Json(json_response))
}

//create the note handler
pub async fn create_note_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_id = uuid::Uuid::new_v4().to_string();

    let query_result =
        sqlx::query(r#"INSERT INTO notes (id,title,content,category) VALUES (?,?,?,?)"#)
            .bind(user_id.clone())
            .bind(body.title.to_string())
            .bind(body.content.to_string())
            .bind(body.category.to_string())
            .execute(&data.db)
            .await
            .map_err(|err: sqlx::Error| err.to_string());

    if let Err(err) = query_result {
        if err.contains("Duplicate entry") {
            let error_response = serde_json::json!({
                "status":"fail",
                "message":"Duplicate entry",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status":"error","message":format!("{:?}",err)})),
        ));
    }
    let note = sqlx::query_as!(NoteModel, r#"SELECT * FROM notes WHERE id = ?"#, user_id)
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status":"error","message":format!("{:?}",e)})),
            )
        })?;
    let note_response = serde_json::json!({
        "status":"success",
        "data":serde_json::json!({
            "note":filter_db_record(&note)
        })
    });

    Ok(Json(note_response))
}


