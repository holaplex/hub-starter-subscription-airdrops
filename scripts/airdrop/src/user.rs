use crate::prelude::*;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: String,
    #[sqlx(rename = "holaplexCustomerId")]
    pub cust_id: Uuid,
    pub wallet: String,
}

#[derive(Debug)]
pub enum UserFindError {
    UserNotFound,
    CustomerIdNotFound,
    WalletNotFound,
    DbError(Box<dyn std::error::Error>),
}

impl std::fmt::Display for UserFindError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UserNotFound => write!(f, "User not found"),
            Self::CustomerIdNotFound => write!(f, "Customer ID not found"),
            Self::WalletNotFound => write!(f, "Wallet not found"),
            Self::DbError(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for UserFindError {}

impl User {
    pub async fn find(db: &crate::PgPool, id: &str) -> Result<Self, UserFindError> {
        let record: Result<Option<(Option<String>, Option<Uuid>, Option<String>)>, _> =
            sqlx::query_as(
                r#"
            SELECT "User"."id", "User"."holaplexCustomerId", "Wallet"."address"
            FROM "User"
            INNER JOIN "Wallet" ON "User"."holaplexCustomerId" = "Wallet"."holaplexCustomerId"
            WHERE "User"."id" = $1
            "#,
            )
            .bind(id)
            .fetch_optional(db)
            .await;

        match record {
            Ok(Some((Some(id), Some(cust_id), Some(wallet)))) => Ok(User {
                id,
                cust_id,
                wallet,
            }),
            Ok(Some((Some(_), Some(_), None))) => Err(UserFindError::WalletNotFound),
            Ok(Some((Some(_), None, _))) => Err(UserFindError::CustomerIdNotFound),
            Ok(Some((None, _, _))) | Ok(None) => Err(UserFindError::UserNotFound),
            Err(e) => Err(UserFindError::DbError(Box::new(e))),
        }
    }
}
