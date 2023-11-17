use axum::{response::IntoResponse,routing::get,Json,Router};



async fn health_checker_handler()->impl IntoResponse{
    const MESSAGE: &str = "RUST/REACT CRUD";
    let json_response = serde_json::json!({
        "status":"success",
        "message":MESSAGE
    });
    Json(json_response)
}
#[tokio::main]
async fn main() {
    let app = Router::new().route("/api/healthchecker",get(health_checker_handler));
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
 
}
