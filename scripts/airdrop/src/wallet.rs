use crate::prelude::*;

#[derive(Debug, FromRow, Serialize)]
pub struct Wallet {
    pub address: Option<String>,
}

impl Wallet {
    pub async fn by_customer_id(db: &PgPool, cust_id: Uuid) -> Result<Option<Wallet>, Error> {
        query_as::<_, Wallet>(
            r#"
        SELECT address
        FROM "Wallet"
        WHERE "holaplexCustomerId" = $1
        "#,
        )
        .bind(cust_id)
        .fetch_optional(db)
        .await
    }
}
