use graphql_client::{GraphQLQuery, Response};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/get_drops.graphql",
    mutation_path = "mutations/mint.graphql",
    response_derives = "Debug,Serialize,Deserialize"
)]
struct GetDropsQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/mint.graphql",
    response_derives = "Debug,Serialize,Deserialize"
)]
struct MintMutation;

#[derive(Debug, Deserialize)]
struct GetDropsData {
    project: Project,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: String,
    drops: Option<Vec<Drop>>,
}

#[derive(Debug, Deserialize)]
struct Drop {
    id: String,
    startTime: Option<String>,
    endTime: Option<String>,
    collection: Option<Collection>,
}

#[derive(Debug, Deserialize)]
struct Collection {
    totalMints: Option<i32>,
    supply: Option<i32>,
    id: Option<String>,
    address: Option<String>,
    holders: Option<Vec<Holder>>,
    metadataJson: Option<MetadataJson>,
}

#[derive(Debug, Deserialize)]
struct Holder {
    address: Option<String>,
    owns: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataJson {
    id: Option<String>,
    image: Option<String>,
    name: Option<String>,
    description: Option<String>,
    attributes: Option<Vec<Attribute>>,
}

#[derive(Debug, Deserialize)]
struct Attribute {
    traitType: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MintNftData {
    mintEdition: Option<MintEditionPayload>,
}

#[derive(Debug, Deserialize)]
struct MintEditionPayload {
    collectionMint: Option<CollectionMint>,
}

#[derive(Debug, Deserialize)]
struct CollectionMint {
    address: Option<String>,
    owner: Option<String>,
}

#[derive(Debug, Serialize)]
struct MintNftVars {
    input: MintDropInput,
}

#[derive(Debug, Serialize)]
struct MintDropInput {
    drop: String,
    recipient: String,
}

#[derive(Debug, Serialize)]
struct GetDropsVars {
    project: String,
}

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap()
        .block_on(start());
}

async fn start() -> Result<(), Box<dyn std::error::Error>> {
    println!("Start Airdrop");
    println!("Project id: {}", std::env::var("HOLAPLEX_PROJECT_ID")?);
    let client = Client::new();
    let variables = GetDropsVars {
        project: std::env::var("HOLAPLEX_PROJECT_ID")?,
    };

    let response_body = client
        .post(process.env.HOLAPLEX_API_ENDPOINT)
        .header("Authorization", process.env.HOLAPLEX_AUTH_TOKEN)
        .json(&GetDropsQuery::build_query(variables))
        .send()?
        .text()?;

    let response_data: Response<get_drops_query::ResponseData> =
        serde_json::from_str(&response_body)?;

    let result = response_data.data.expect("No response data");

    if let Some(drops) = result.project.drops {
        for drop in drops {
            println!("{:?}", drop);
            if drop.startTime.is_none() || drop.startTime.unwrap() <= chrono::Utc::now().to_rfc3339() {
                println!("Drop open for minting: {}", drop.id);
            }
            if let Some(airdrop) = db::airdrop.find_first(drop.id) {
                println!("airdrop: {:?}", airdrop);
            }
            if airdrop.is_none() || airdrop.unwrap().completedAt.is_none() {
                let update_start_time = db::airdrop.upsert(drop.id, new Date());
                println!("Add starttime: {:?}", update_start_time);

                println!("Start minting");
                mint(drop.id).await?;

                let update_end_time = db::airdrop.upsert(drop.id, new Date());
                println!("Add endTime: {:?}", update_end_time);
            }
        }
    }
    Ok(())
}

async fn mint(drop_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let subscriptions = db::subscription.find_many().await?;
    println!("Subscriptions: {:?}", subscriptions);
    for sub in subscriptions {
        if let Some(wallet) = db::wallet.find_first(sub.user.id).await? {
            println!("Wallet: {:?}", wallet);
            let variables = MintNftVars {
                input: MintDropInput {
                    drop: drop_id.clone(),
                    recipient: wallet.address.clone(),
                },
            };
            let response_body = client
                .post(process.env.HOLAPLEX_API_ENDPOINT)
                .header("Authorization", process.env.HOLAPLEX_AUTH_TOKEN)
                .json(&MintMutation::build_query(variables))
                .send()?
                .text()?;
            
            let response_data: Response<mint_nft_mutation::ResponseData> =
                serde_json::from_str(&response_body)?;
            let result = response_data.data.expect("No response data");
            println!("Mint result: {:?}", result.mintEdition);
        }
    }
    Ok(())
}
