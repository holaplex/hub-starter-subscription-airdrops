use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Airdrop {
    pub drop_id: String,
    pub completed_at: Option<NaiveDateTime>,
}

pub async fn find_airdrop_by_drop_id(
    pool: &sqlx::PgPool,
    drop_id: &str,
) -> Result<Option<Airdrop>, sqlx::Error> {
    sqlx::query_as::<_, Airdrop>(
        r#"
        SELECT *
        FROM "Airdrop"
        WHERE "dropId" = $1
        "#,
    )
    .bind(drop_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_airdrop(
    pool: &sqlx::PgPool,
    drop_id: &str,
    started_at: Option<NaiveDateTime>,
    completed_at: Option<NaiveDateTime>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO "Airdrop" ("dropId", "startedAt", "completedAt")
        VALUES ($1, $2, $3)
        ON CONFLICT ("dropId")
        DO UPDATE SET "startedAt" = EXCLUDED."startedAt", "completedAt" = EXCLUDED."completedAt"
        "#,
        drop_id,
        started_at,
        completed_at
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Subscription {
    pub user_id: i32,
    pub subscribed_at: Option<chrono::NaiveDateTime>,
}

pub async fn find_subscriptions(pool: &sqlx::PgPool) -> Result<Vec<Subscription>, sqlx::Error> {
    sqlx::query_as::<_, Subscription>(
        r#"
        SELECT *
        FROM "Subscription"
        WHERE "subscribedAt" IS NOT NULL
        "#,
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Wallet {
    pub address: Option<String>,
}

pub async fn find_wallet_by_user_id(
    pool: &sqlx::PgPool,
    user_id: i32,
) -> Result<Option<Wallet>, sqlx::Error> {
    sqlx::query_as::<_, Wallet>(
        r#"
        SELECT *
        FROM "Wallet"
        WHERE "holaplexCustomerId" = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
}
