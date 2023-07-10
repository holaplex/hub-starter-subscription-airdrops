use crate::prelude::*;

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
