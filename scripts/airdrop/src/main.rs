use chrono::Utc;
use graphql_client::GraphQLQuery;
use reqwest::{header, Url};
use sqlx::PgPool;
use std::env;

mod db;

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
    response_derives = "Debug"
)]

struct MintEdition;

#[allow(clippy::upper_case_acronyms)]
type UUID = uuid::Uuid;

type DateTime = chrono::DateTime<Utc>;

struct Context {
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
    dotenv::vars();
    let context = Context::new().await?;
    let project_id = UUID::parse_str(&env::var("HOLAPLEX_PROJECT_ID")?)?;
    start(&context, project_id).await
}

async fn start(ctx: &Context, project_id: UUID) -> Result<(), Box<dyn std::error::Error>> {
    println!("Start Airdrop");
    let query = &GetDrops::build_query(get_drops::Variables {
        project: project_id,
    });
    let drops_result = ctx
        .gql_client
        .post(ctx.api_endpoint.clone())
        .json(query)
        .send()
        .await?;

    let drops_response: graphql_client::Response<get_drops::ResponseData> =
        drops_result.json().await?;

    if let Some(drops) = drops_response
        .data
        .and_then(|data| data.project)
        .and_then(|project| project.drops)
    {
        for drop in drops {
            println!("Got drops:\n {}", serde_json::to_string_pretty(&drop)?);
            let airdrop = db::find_airdrop_by_drop_id(&ctx.db_pool, &drop.id.to_string()).await?;

            println!("airdrop {:?}", airdrop);

            if let Some(airdrop) = airdrop {
                if drop.start_time.is_none() || drop.start_time.unwrap() <= Utc::now() {
                    println!("Drop open for minting: {:?}", drop.id);
                }

                if airdrop.completed_at.is_none() {
                    // Set Airdrop starttime
                    db::upsert_airdrop(
                        &ctx.db_pool,
                        &airdrop.drop_id,
                        Some(Utc::now().naive_local()),
                        None,
                    )
                    .await?;
                    println!("Start time added");

                    // Airdrop
                    println!("Start minting");
                    if let Err(e) = mint(ctx, &drop.id.to_string()).await {
                        println!("Error in minting: {:?}", e);
                        continue;
                    }

                    // Set Airdrop endtime
                    db::upsert_airdrop(
                        &ctx.db_pool,
                        &airdrop.drop_id,
                        None,
                        Some(Utc::now().naive_local()),
                    )
                    .await?;
                    println!("End time added");
                }
            }
        }
    }

    Ok(())
}

async fn mint(ctx: &Context, drop_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let subscriptions = db::find_subscriptions(&ctx.db_pool).await?;
    println!("Subscriptions: {:?}", subscriptions);

    for subscription in subscriptions {
        let wallet = db::find_wallet_by_user_id(&ctx.db_pool, subscription.user_id).await?;
        println!("Wallet: {:?}", wallet);
        if let Some(wallet) = wallet {
            let mutation = MintEdition::build_query(mint_edition::Variables {
                input: mint_edition::MintDropInput {
                    drop: UUID::parse_str(drop_id)?,
                    recipient: wallet.address.unwrap_or_default(),
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

            println!("Mint result: {:?}", mint_result.data);
        }
    }

    Ok(())
}
