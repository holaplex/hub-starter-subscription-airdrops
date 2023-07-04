use crate::get_drops::GetDropsProjectDrops;
use chrono::Utc;
use db::{Airdrop, Subscription};
use graphql_client::GraphQLQuery;
use log::{error, info, warn};
use reqwest::{header, Url};
use sqlx::PgPool;
use std::env;
use user::User;
mod db;
mod user;

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
    start(&context, project_id).await
}

async fn start(ctx: &Context, project_id: UUID) -> Result<(), Box<dyn std::error::Error>> {
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
            warn!("Drop start_time not reached. skipping");
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
            Ok(Some(user)) => {
                info!("Found user: {:?}", user);

                if let Some(recipient) = user.wallet {
                    info!("Found wallet for user: {}", recipient);

                    if let Some(purchases) = &drop.purchases {
                        if purchases.iter().any(|p| p.wallet == recipient) {
                            warn!("User already received airdrop. Skipping");
                            continue;
                        }
                    }

                    let mutation = MintEdition::build_query(mint_edition::Variables {
                        input: mint_edition::MintDropInput {
                            drop: drop.id,
                            recipient,
                        },
                    });
                    let result = ctx
                        .gql_client
                        .post(ctx.api_endpoint.clone())
                        .json(&mutation)
                        .send()
                        .await?;

                    let mint_result: graphql_client::Response<mint_edition::ResponseData> =
                        result.json().await?;
                    info!("Mint result: {:?}", mint_result.data);
                } else {
                    error!(
                        "Found user but wallet address was not found for user_id: {}",
                        user.user_id
                    );
                }
            }
            Ok(None) => error!(
                "Unable to find Holaplex customer id for user_id: {}",
                subscription.user_id
            ),
            Err(e) => error!("Error while finding user: {}", e),
        }
    }

    Ok(())
}
