use graphql_client::GraphQLQuery;
pub use prelude::*;
use reqwest::{header, Url};
use serde_json::Value;
use std::env;
pub use {
    airdrop::Airdrop,
    get_drops::GetDropsProjectDrops,
    subscription::Subscription,
    user::{User, UserFindError},
    wallet::Wallet,
};
mod airdrop;
mod prelude;
mod subscription;
mod user;
mod wallet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "holaplex.graphql",
    query_path = "queries/get_drops.graphql",
    response_derives = "Debug, Serialize"
)]
struct GetDrops;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "holaplex.graphql",
    query_path = "queries/mint_edition.graphql",
    response_derives = "Debug, Serialize"
)]
struct MintEdition;

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
    path: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}
#[allow(clippy::upper_case_acronyms)]
pub type UUID = uuid::Uuid;

type DateTime = chrono::DateTime<Utc>;

pub struct Context {
    gql_client: reqwest::Client,
    api_endpoint: Url,
    db_pool: PgPool,
}

impl Context {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_endpoint = Url::parse(&env::var("HOLAPLEX_API_ENDPOINT")?)?;
        let auth_token = env::var("HOLAPLEX_AUTH_TOKEN")?;

        let mut headers = header::HeaderMap::new();
        let header_value =
            header::HeaderValue::from_str(&auth_token).map_err(|_| "Invalid header value")?;
        headers.insert("Authorization", header_value);

        let gql_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        let db_pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;

        Ok(Context {
            gql_client,
            api_endpoint,
            db_pool,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv::vars();
    let context = Context::new().await?;
    let project_id = UUID::parse_str(&env::var("HOLAPLEX_PROJECT_ID")?)?;
    run(&context, project_id).await
}

async fn run(ctx: &Context, project_id: UUID) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Airdrop");
    let query = GetDrops::build_query(get_drops::Variables {
        project: project_id,
    });

    let response: graphql_client::Response<get_drops::ResponseData> = ctx
        .gql_client
        .post(ctx.api_endpoint.clone())
        .json(&query)
        .send()
        .await?
        .json()
        .await?;

    let drops = response
        .data
        .ok_or("missing response data")?
        .project
        .ok_or("missing project")?
        .drops
        .ok_or("missing drops")?;

    for drop in drops {
        if !(drop.start_time.is_none() || drop.start_time.unwrap() <= Utc::now()) {
            info!("Drop start_time not reached. Skipping");
            continue;
        };

        let mut airdrop = Airdrop::find_by_drop_id(&ctx.db_pool, drop.id.to_string())
            .await?
            .unwrap_or_else(|| Airdrop::new(drop.id));

        if airdrop.completed_at.is_some() {
            warn!(
                "Airdrop for Drop ID: {} completed at {}. Skipping",
                airdrop.drop_id,
                airdrop.completed_at.unwrap()
            );
            continue;
        }

        // Set Airdrop starttime
        log::info!("Starting airdrop for drop: {}", drop.id);
        airdrop.update(&ctx.db_pool).await?;

        // Airdrop mint
        match mint(ctx, &drop).await {
            Ok(_) => {
                // Set Airdrop endtime
                airdrop.completed_at = Some(Utc::now().naive_local());
                airdrop.update(&ctx.db_pool).await?;
                info!("Airdrop completed at: {}", airdrop.completed_at.unwrap());
            }
            Err(e) => {
                error!("Error while minting: {:?}", e);
            }
        }
    }

    Ok(())
}

async fn mint(
    ctx: &Context,
    drop: &GetDropsProjectDrops,
) -> Result<(), Box<dyn std::error::Error>> {
    let subscriptions = Subscription::fetch(&ctx.db_pool).await?;
    info!("Retrieved {} subscriptions.", subscriptions.len());

    for subscription in &subscriptions {
        match User::find(&ctx.db_pool, &subscription.user_id).await {
            Ok(user) => {
                if drop
                    .purchases
                    .as_ref()
                    .map(|p| p.iter().any(|p| p.wallet == user.wallet))
                    .unwrap_or(false)
                {
                    info!("Wallet {} already received airdrop. Skipping", user.wallet);
                    continue;
                }

                let mutation = MintEdition::build_query(mint_edition::Variables {
                    input: mint_edition::MintDropInput {
                        drop: drop.id,
                        recipient: user.wallet,
                    },
                });

                match ctx
                    .gql_client
                    .post(ctx.api_endpoint.clone())
                    .json(&mutation)
                    .send()
                    .await
                {
                    Ok(response) => {
                        let res: GraphQLResponse<Value> = response.json().await?;
                        if let Some(errors) = res.errors {
                            for error in errors {
                                error!("Error: {}\nPath: {:?}", error.message, error.path);
                            }
                            return Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "GraphQL error.",
                            )));
                        } else {
                            info!("Success: {}", res.data.unwrap().to_string())
                        }
                    }
                    Err(e) => {
                        error!("Failed to send request to GraphQL client: {}", e);
                        return Err(Box::new(e));
                    }
                }
            }
            Err(e) => match e {
                UserFindError::DbError(e) => {
                    error!("Database error: {}", e);
                    return Err(e);
                }
                UserFindError::UserNotFound => {
                    warn!("User {} not found. Skipping", subscription.user_id);
                    continue;
                }
                UserFindError::CustomerIdNotFound => {
                    warn!(
                        "Customer id not found for user_id: {}. Skipping",
                        subscription.user_id
                    );
                    continue;
                }
                UserFindError::WalletNotFound => {
                    warn!(
                        "Wallet not found for user_id: {}. Skipping",
                        subscription.user_id
                    );
                    continue;
                }
            },
        }
    }

    Ok(())
}
