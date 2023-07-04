use chrono::Utc;
use serde::Serialize;
use sqlx::{query, query_as, types::chrono::NaiveDateTime, Error, FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Airdrop {
    #[sqlx(rename = "dropId")]
    pub drop_id: String,
    #[sqlx(rename = "startedAt")]
    pub started_at: Option<NaiveDateTime>,
    #[sqlx(rename = "completedAt")]
    pub completed_at: Option<NaiveDateTime>,
}

impl Airdrop {
    pub fn new(drop_id: Uuid) -> Self {
        Self {
            drop_id: drop_id.to_string(),
            started_at: Some(Utc::now().naive_local()),
            completed_at: None,
        }
    }
    pub async fn find_by_drop_id(db: &PgPool, drop_id: String) -> Result<Option<Airdrop>, Error> {
        query_as::<_, Airdrop>(
            r#"
        SELECT *
        FROM "Airdrop"
        WHERE "dropId" = $1
        "#,
        )
        .bind(drop_id)
        .fetch_optional(db)
        .await
    }
    pub async fn update(&self, db: &PgPool) -> Result<(), Error> {
        query!(
            r#"
        INSERT INTO "Airdrop" ("dropId", "startedAt", "completedAt")
        VALUES ($1, $2, $3)
        ON CONFLICT ("dropId")
        DO UPDATE SET "startedAt" = EXCLUDED."startedAt", "completedAt" = EXCLUDED."completedAt"
        "#,
            self.drop_id.to_string(),
            self.started_at,
            self.completed_at
        )
        .execute(db)
        .await?;
        Ok(())
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct Subscription {
    #[sqlx(rename = "userId")]
    pub user_id: String,
    #[sqlx(rename = "subscribedAt")]
    pub subscribed_at: Option<chrono::NaiveDateTime>,
}

impl Subscription {
    pub async fn fetch(db: &PgPool) -> Result<Vec<Subscription>, Error> {
        query_as::<_, Subscription>(
            r#"
        SELECT "userId", "subscribedAt"
        FROM "Subscription"
        WHERE "subscribedAt" IS NOT NULL
        "#,
        )
        .fetch_all(db)
        .await
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct Wallet {
    pub address: Option<String>,
}
impl Wallet {
    pub async fn find_by_customer_id(
        db: &PgPool,
        customer_id: Uuid,
    ) -> Result<Option<Wallet>, Error> {
        query_as::<_, Wallet>(
            r#"
        SELECT address
        FROM "Wallet"
        WHERE "holaplexCustomerId" = $1
        "#,
        )
        .bind(customer_id)
        .fetch_optional(db)
        .await
    }
}

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    #[sqlx(rename = "holaplexCustomerId")]
    pub holaplex_customer_id: Option<Uuid>,
}

impl User {
    pub async fn find_by_customer_id(db: &PgPool, user_id: &str) -> Result<Option<User>, Error> {
        query_as::<_, User>(
            r#"
        SELECT "holaplexCustomerId"
        FROM "User"
        WHERE "id" = $1
        "#,
        )
        .bind(user_id)
        .fetch_optional(db)
        .await
    }
}
