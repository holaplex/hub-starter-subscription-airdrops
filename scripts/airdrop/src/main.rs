use chrono::Utc;
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use reqwest::Url;
use std::env;
use sqlx::PgPool;
use tokio::time::{sleep, Duration};
use serde_json::json;

mod db;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "holaplex.graphql",
    query_path = "queries/get_drops.graphql",
    response_derives = "Debug"
)]
struct GetDropsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/mint_edition.graphql",
    response_derives = "Debug"
)]
struct MintEditionMutation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the GraphQL client
    let graphql_endpoint = env::var("GRAPHQL_ENDPOINT").expect("GRAPHQL_ENDPOINT not set");
    let graphql_auth_token = env::var("GRAPHQL_AUTH_TOKEN").expect("GRAPHQL_AUTH_TOKEN not set");
    let graphql_client = reqwest::Client::new();
    let graphql_url = Url::parse(&graphql_endpoint)?;
    let db_pool = PgPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL not set")).await?;

    // Call the start function
    start(&graphql_client, &graphql_url, &db_pool).await?;

    Ok(())
}

async fn start(
    graphql_client: &reqwest::Client,
    graphql_url: &Url,
    db_pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Start Airdrop");
    let project_id = env::var("HOLAPLEX_PROJECT_ID").expect("HOLAPLEX_PROJECT_ID not set");

    let drops_result = graphql_client
        .post(graphql_url.clone())
        .header("Authorization", &env::var("GRAPHQL_AUTH_TOKEN").expect("GRAPHQL_AUTH_TOKEN not set"))
        .body(&GetDropsQuery::build_query(get_drops_query::Variables {
            project: project_id,
        }))
        .send()
        .await?;


    let drops_response: graphql_client::Response<get_drops_query::ResponseData> =
        drops_result.json().await?;

    if let Some(drops) = drops_response
        .data
        .and_then(|data| data.project)
        .and_then(|project| project.drops)
    {
        for drop in drops {
            println!("{:?}", drop);
            let airdrop = db::find_airdrop_by_drop_id(&db_pool, &drop.id).await?;

            println!("airdrop {:?}", airdrop);

            if let Some(airdrop) = airdrop {
                if drop.start_time.is_none() || drop.start_time.unwrap() <= Utc::now().naive_local() {
                    println!("Drop open for minting: {:?}", drop.id);
                }

                if airdrop.completed_at.is_none() {
                    // Set Airdrop starttime
                    let _ = db::upsert_airdrop(
                        &db_pool,
                        &airdrop.drop_id,
                        Some(Utc::now().naive_local()),
                        None,
                    )
                    .await?;
                    println!("Add starttime: {:?}", update_start_time);

                    // Airdrop
                    println!("Start minting");
                    mint(&db_pool, &drop.id).await?;

                    // Set Airdrop endtime
                    let _ = db::upsert_airdrop(
                        &db_pool,
                        &airdrop.drop_id,
                        None,
                        Some(Utc::now().naive_local()),
                    )
                    .await?;
                    println!("Add endTime: {:?}", update_end_time);
                }
            }
        }
    }

    Ok(())
}

async fn mint(db_pool: &PgPool, drop_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let subscriptions = db::find_subscriptions(&db_pool).await?;
    println!("Subscriptions: {:?}", subscriptions);

    for subscription in subscriptions {
        let wallet = db::find_wallet_by_user_id(&db_pool, subscription.user_id).await?;
        println!("Wallet: {:?}", wallet);

        let result = graphql_client
            .post(graphql_url.clone())
            .header("Authorization", &env::var("GRAPHQL_AUTH_TOKEN").expect("GRAPHQL_AUTH_TOKEN not set"))
            .body(&MintEditionMutation::build_query(mint_edition_mutation::Variables {
                input: mint_edition_mutation::MintDropInput {
                    drop: drop_id.to_owned(),
                    recipient: wallet.unwrap_or_default().address,
                },
            }))
            .send()
            .await?;

        let mint_result: graphql_client::Response<mint_edition_mutation::ResponseData> =
            result.json().await?;

        println!("Mint result: {:?}", mint_result.data);
    }

    Ok(())
}
