use std::{str::FromStr, time::Duration};

use chrono::{NaiveDateTime, Utc};
use serde_json::json;
use sqlx::{
    prelude::FromRow,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    Pool, Sqlite,
};

use crate::{PostTransaction, TransactionKind};

#[derive(Clone)]
pub struct SqliteRepository {
    pool: Pool<Sqlite>,
}

#[derive(FromRow)]
pub struct Client {
    pub id: i32,
    pub limit: i32,
    pub balance: i32,
}

#[derive(Debug, FromRow)]
pub struct Transaction {
    pub value: i32,
    pub kind: String,
    pub description: String,
    pub created_at: NaiveDateTime,
}

impl SqliteRepository {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let pool_timeout = Duration::from_secs(30);

        let connection_options = SqliteConnectOptions::from_str(&url)?
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(pool_timeout);

        let pool = SqlitePoolOptions::new()
            .max_connections(100)
            .connect_with(connection_options)
            .await?;

        Ok(Self { pool })
    }

    pub async fn extract(&self, client_id: u8) -> Result<serde_json::Value, sqlx::Error> {
        let client: Option<Client> = sqlx::query_as(
            r#"
        SELECT "id", "limit", "balance"
        FROM "clients"
        WHERE "id" = $1
        "#,
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap();

        if let Some(client) = client {
            let extract_json = json!({
              "total": client.balance,
              "data_extrato": Utc::now(),
              "limite": client.limit,}
            );

            let transactions: Vec<Transaction> = sqlx::query_as(
                r#"
                SELECT "value", "kind", "description", "created_at"
                FROM "transactions"
                WHERE "client_id" = $1
                ORDER BY "created_at" DESC
                LIMIT 10
                "#,
            )
            .bind(client_id)
            .fetch_all(&self.pool)
            .await
            .unwrap();

            let mut transactions_json = Vec::new();

            for transaction in transactions.iter() {
                transactions_json.push(json!({
                  "valor": transaction.value,
                  "tipo": transaction.kind,
                  "descricao": &transaction.description,
                  "realizada_em": transaction.created_at,
                }));
            }

            return Ok(json!({
              "saldo": extract_json,
              "ultimas_transacoes": transactions_json}
            ));
        }

        Err(sqlx::Error::RowNotFound)
    }

    pub async fn transact(
        &self,
        client_id: u8,
        transaction_data: PostTransaction,
    ) -> Result<Client, sqlx::Error> {
        let mut transaction = self.pool.begin().await.unwrap();

        let client: Option<Client> = sqlx::query_as(
            r#"
        SELECT "id", "limit", "balance"
        FROM "clients"
        WHERE "id" = $1
        "#,
        )
        .bind(client_id)
        .fetch_optional(&mut *transaction)
        .await
        .unwrap();

        if let Some(mut client) = client {
            let transaction_kind = match transaction_data.kind {
                TransactionKind::Credit => {
                    client.balance += transaction_data.value;
                    "c"
                }
                TransactionKind::Debit => {
                    if client.balance + client.limit < transaction_data.value {
                        return Err(sqlx::Error::WorkerCrashed);
                    }
                    client.balance -= transaction_data.value;
                    "d"
                }
            };
            sqlx::query(r#"UPDATE "clients" SET "balance" = ? WHERE "id" = ?"#)
                .bind(client.balance)
                .bind(client_id)
                .execute(&mut transaction)
                .await
                .unwrap();

            sqlx::query(
                r#"
                INSERT INTO "transactions" (
                  "kind", "value", "description", "client_id", "created_at"
                )
                VALUES (
                  $1, $2, $3, $4, $5
                )
                "#,
            )
            .bind(transaction_kind)
            .bind(transaction_data.value)
            .bind(&transaction_data.description.0)
            .bind(client.id)
            .bind(Utc::now().naive_utc())
            .execute(&mut *transaction)
            .await
            .unwrap();

            transaction.commit().await.unwrap();

            return Ok(client);
        }

        Err(sqlx::Error::RowNotFound)
    }
}
