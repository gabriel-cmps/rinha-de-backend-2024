use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;


use crate::{AppState, PostTransaction};

pub async fn post_transaction(
    Path(client_id): Path<u8>,
    State(state): State<AppState>,
    Json(transaction): Json<PostTransaction>,
) -> impl IntoResponse {
    let transact = state.repository.transact(client_id, transaction).await;

    match transact {
        Ok(client) => Ok(Json(json!({
          "limite": client.limit,
          "saldo": client.balance,
        }))),
        Err(sqlx::Error::RowNotFound) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
}

pub async fn get_extract(
    Path(client_id): Path<u8>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let extract = state.repository.extract(client_id).await;

    match extract {
        Ok(extract) => Ok(Json(extract)),
        Err(_) => Err(StatusCode::NOT_FOUND),
        
    }
}
