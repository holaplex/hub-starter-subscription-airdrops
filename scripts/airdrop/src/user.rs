use crate::db::{self, Wallet};
use uuid::Uuid;

#[derive(Debug)]
pub struct User {
    pub user_id: String,
    pub customer_id: Uuid,
    pub wallet: Option<String>,
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
    pub async fn find(db: &crate::PgPool, user_id: &str) -> Result<Option<Self>, UserFindError> {
        let user = db::User::find_by_customer_id(db, user_id)
            .await
            .map_err(|e| UserFindError::DbError(Box::new(e)))?
            .ok_or(UserFindError::UserNotFound)?;

        let customer_id = user
            .holaplex_customer_id
            .ok_or(UserFindError::CustomerIdNotFound)?;

        let wallet = Wallet::find_by_customer_id(db, customer_id)
            .await
            .map_err(|e| UserFindError::DbError(Box::new(e)))?
            .ok_or(UserFindError::WalletNotFound)?;

        Ok(Some(User {
            user_id: user_id.to_string(),
            customer_id: user.holaplex_customer_id.unwrap(),
            wallet: wallet.address,
        }))
    }
}
