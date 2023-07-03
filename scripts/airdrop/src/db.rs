use sqlx::postgres::PgPool;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Airdrop {
    pub id: i32,
    pub drop_id: String,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

pub async fn find_airdrop_by_drop_id(
    db_pool: &PgPool,
    drop_id: &str,
) -> Result<Option<Airdrop>, sqlx::Error> {
    sqlx::query_as!(
        Airdrop,
        r#"
        SELECT *
        FROM airdrop
        WHERE drop_id = $1
        "#,
        drop_id
    )
    .fetch_optional(db_pool)
    .await
}

pub async fn upsert_airdrop(
    db_pool: &PgPool,
    drop_id: &str,
    started_at: Option<NaiveDateTime>,
    completed_at: Option<NaiveDateTime>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO airdrop (drop_id, started_at, completed_at)
        VALUES ($1, $2, $3)
        ON CONFLICT (drop_id)
        DO UPDATE SET started_at = EXCLUDED.started_at, completed_at = EXCLUDED.completed_at
        "#,
        drop_id,
        started_at,
        completed_at
    )
    .execute(db_pool)
    .await?;
    Ok(())
}

#[derive(Debug, FromRow)]
pub struct Subscription {
    pub id: i32,
    pub user_id: i32,
}

pub async fn find_subscriptions(
    db_pool: &PgPool,
) -> Result<Vec<Subscription>, sqlx::Error> {
    sqlx::query_as!(
        Subscription,
        r#"
        SELECT *
        FROM subscription
        WHERE subscribed_at IS NOT NULL
        "#,
    )
    .fetch_all(db_pool)
    .await
}

#[derive(Debug, FromRow)]
pub struct Wallet {
    pub id: i32,
    pub address: Option<String>,
}

pub async fn find_wallet_by_user_id(
    db_pool: &PgPool,
    user_id: i32,
) -> Result<Option<Wallet>, sqlx::Error> {
    sqlx::query_as!(
        Wallet,
        r#"
        SELECT *
        FROM wallet
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(db_pool)
    .await
}
