use axum::{
    routing::{get, post},
    Router,
};
use database::SqliteRepository;
use handlers::{get_extract, post_transaction};
use std::env;
use serde::{Deserialize, Serialize};

pub mod handlers;
pub mod database;

#[derive(Clone)]
pub struct AppState {
    repository: SqliteRepository,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum TransactionKind {
    #[serde(rename = "c")]
    Credit,
    #[serde(rename = "d")]
    Debit,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct TransactionDescription(pub String);

impl TryFrom<String> for TransactionDescription {
    type Error = &'static str;

    fn try_from(description: String) -> Result<Self, Self::Error> {
        if description.is_empty() || description.len() > 10 {
            return Err("Invalid description");
        }
        Ok(Self(description))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PostTransaction {
    #[serde(rename = "valor")]
    value: i32,
    #[serde(rename = "tipo")]
    kind: TransactionKind,
    #[serde(rename = "descricao")]
    description: TransactionDescription,
}


#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(3000);

    let url = env::var("DB")
        .unwrap_or(String::from("sqlite:db/db.sqlite"));

    dbg!(&url);

    let repository = SqliteRepository::new(&url).await.unwrap();

    let state = AppState { repository };

    let app = Router::new()
        .route("/clientes/:id/transacoes", post(post_transaction))
        .route("/clientes/:id/extrato", get(get_extract))
        .with_state(state);

    println!("Server running on port {port}");
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
