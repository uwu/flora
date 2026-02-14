use crate::{
    api_types::{
        CreateStoreRequest, CreateStoreResponse, GetValueResponse, KvStore, ListKeysResponse,
        SetValueRequest,
    },
    auth::AuthRequestBuilder,
    cli::KvCommands,
    config::CliConfig,
};
use color_eyre::eyre::{Result, eyre};
use reqwest::Client;

pub(crate) async fn handle_kv_command(
    client: &Client,
    config: &CliConfig,
    cmd: KvCommands,
) -> Result<()> {
    match cmd {
        KvCommands::CreateStore { guild, name } => {
            let url = format!("{}/kv/stores", config.api_url);
            let body = CreateStoreRequest {
                guild_id: guild.clone(),
                store_name: name.clone(),
            };
            let resp: CreateStoreResponse = client
                .post(url)
                .maybe_bearer(&config.token)?
                .json(&body)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            println!(
                "Created KV store '{}' for guild {}",
                resp.store.store_name, resp.store.guild_id
            );
        }
        KvCommands::ListStores { guild } => {
            let url = format!("{}/kv/stores?guild_id={}", config.api_url, guild);
            let stores: Vec<KvStore> = client
                .get(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if stores.is_empty() {
                println!("No KV stores found for guild {}", guild);
            } else {
                println!("KV stores for guild {}:", guild);
                for store in stores {
                    println!("  - {}", store.store_name);
                }
            }
        }
        KvCommands::DeleteStore { guild, name } => {
            let url = format!("{}/kv/stores/{}/{}", config.api_url, guild, name);
            client
                .delete(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?;
            println!("Deleted KV store '{}' for guild {}", name, guild);
        }
        KvCommands::Set {
            guild,
            store,
            key,
            value,
            expiration,
            metadata,
        } => {
            let url = format!("{}/kv/{}/{}/{}", config.api_url, guild, store, key);
            let metadata_json: Option<serde_json::Value> = match metadata {
                Some(ref m) => Some(
                    serde_json::from_str(m).map_err(|e| eyre!("Invalid JSON metadata: {}", e))?,
                ),
                None => None,
            };
            let body = SetValueRequest {
                value: value.clone(),
                expiration,
                metadata: metadata_json,
            };
            client
                .put(url)
                .maybe_bearer(&config.token)?
                .json(&body)
                .send()
                .await?
                .error_for_status()?;
            let mut msg = format!(
                "Set {}={} in store '{}' for guild {}",
                key, value, store, guild
            );
            if let Some(exp) = expiration {
                msg.push_str(&format!(" (expires: {})", exp));
            }
            println!("{}", msg);
        }
        KvCommands::Get { guild, store, key } => {
            let url = format!("{}/kv/{}/{}/{}", config.api_url, guild, store, key);
            let resp: GetValueResponse = client
                .get(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            match resp.value {
                Some(value) => println!("{}", value),
                None => println!("Key '{}' not found", key),
            }
        }
        KvCommands::Delete { guild, store, key } => {
            let url = format!("{}/kv/{}/{}/{}", config.api_url, guild, store, key);
            client
                .delete(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?;
            println!(
                "Deleted key '{}' from store '{}' for guild {}",
                key, store, guild
            );
        }
        KvCommands::ListKeys {
            guild,
            store,
            prefix,
            limit,
        } => {
            let mut url = format!("{}/kv/{}/{}", config.api_url, guild, store);
            let mut params = Vec::new();
            if let Some(p) = prefix {
                params.push(format!("prefix={}", p));
            }
            if let Some(l) = limit {
                params.push(format!("limit={}", l));
            }
            if !params.is_empty() {
                url = format!("{}?{}", url, params.join("&"));
            }
            let resp: ListKeysResponse = client
                .get(url)
                .maybe_bearer(&config.token)?
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if resp.keys.is_empty() {
                println!("No keys found in store '{}'", store);
            } else {
                println!(
                    "Keys in store '{}' ({} of {} shown):",
                    store,
                    resp.keys.len(),
                    if resp.list_complete { "all" } else { "partial" }
                );
                for key in &resp.keys {
                    let mut line = format!("  - {}", key.name);
                    if let Some(exp) = key.expiration {
                        line.push_str(&format!(" (expires: {})", exp));
                    }
                    if let Some(ref meta) = key.metadata {
                        line.push_str(&format!(" [metadata: {}]", meta));
                    }
                    println!("{}", line);
                }
                if !resp.list_complete {
                    println!(
                        "\nMore keys available. Use --limit {} to get more, or --cursor {} for next page.",
                        limit.unwrap_or(100),
                        resp.cursor.clone().unwrap_or_default()
                    );
                }
            }
        }
    }
    Ok(())
}
