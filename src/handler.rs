use std::sync::Arc;
use pwhash::bcrypt;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use sqlx::query;

use crate::{
    model::{NoteModel, NoteModelResponse,UserModel},
    schema::{CreateNoteSchema, FilterOptions, ParamOptions, SaveUser, UpdateNoteSchema},
    AppState,
};

fn filter_db_record(note: &NoteModel) -> NoteModelResponse {
    NoteModelResponse {
        id: note.id.to_owned(),
        title: note.title.to_owned(),
        content: note.content.to_owned(),
        category: note.category.to_owned(),
        published: note.published != 0,
        created_at: note.created_at.to_owned(),
        updated_at: note.updated_at.to_owned(),
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
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let note_responses = notes
        .iter()
        .map(|note| filter_db_record(&note))
        .collect::<Vec<NoteModelResponse>>();

    let json_response = serde_json::json!({
        "status": "success",
        "results": note_responses.len(),
        "notes": note_responses
    });

    Ok(Json(json_response))
}

//create the user
pub async fn save_user_info(
    State(data):State<Arc<AppState>>,
    Json(body):Json<SaveUser>
)->Result<impl IntoResponse,(StatusCode,Json<serde_json::Value>)>{

    let user_id = uuid::Uuid::new_v4().to_string();
    let password = bcrypt::hash(body.password.to_string()).unwrap();
    
    let query_result = sqlx::query(r#"INSERT INTO users (id,first_name,second_name,email,password,dob) VALUES(?,?,?,?,?,?,?)"#)
    .bind(user_id.clone())
    .bind(body.first_name.to_string())
    .bind(body.second_name.to_string())
    .bind(body.email.to_string())
    .bind(password.clone())
    .execute(&data.db)
    .await
    .map_err(|err: sqlx::Error| err.to_string() );

if let Err(err) = query_result{
    if err.contains("Duplicate entry"){
        let error_response = serde_json::json!({
            "status":"fail",
            "message":"Duplicate entry",
        });
        return Err((StatusCode::CONFLICT,Json(error_response)));
    }
    return Err((StatusCode::INTERNAL_SERVER_ERROR,Json(json!({"status":"error","message":format!("{:?}",err)}))));
}

let reg_response = serde_json::json!({
    "status":"success",
    "message":"User created successfully",

});

Ok(Json(reg_response))


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
            .bind(body.category.to_owned().unwrap_or_default())
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

pub async fn get_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes WHERE id = ?"#,
        id.to_string()
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": filter_db_record(&note)
            })});

            return Ok(Json(note_response));
        }
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };
}

pub async fn edit_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes WHERE id = ?"#,
        id.to_string()
    )
    .fetch_one(&data.db)
    .await;

    let note = match query_result {
        Ok(note) => note,
        Err(sqlx::Error::RowNotFound) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Note with ID: {} not found", id)
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)));
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    };

    let published = body.published.unwrap_or(note.published != 0);
    let i8_publised = published as i8;

    let update_result = sqlx::query(
        r#"UPDATE notes SET title = ?, content = ?, category = ?, published = ? WHERE id = ?"#,
    )
    .bind(body.title.to_owned())
    .bind(
        body.content
            .to_owned(),
    )
    .bind(
        body.category
            .to_owned()
            .unwrap_or_else(|| note.category.clone().unwrap()),
    )
    .bind(i8_publised)
    .bind(id.to_string())
    .execute(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    if update_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let updated_note = sqlx::query_as!(
        NoteModel,
        r#"SELECT * FROM notes WHERE id = ?"#,
        id.to_string()
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"status": "error","message": format!("{:?}", e)})),
        )
    })?;

    let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
        "note": filter_db_record(&updated_note)
    })});

    Ok(Json(note_response))
}

pub async fn delete_note_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query!(r#"DELETE FROM notes WHERE id = ?"#, id.to_string())
        .execute(&data.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            )
        })?;

    if query_result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    Ok(StatusCode::NO_CONTENT)
}

