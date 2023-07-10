use crate::prelude::*;

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
