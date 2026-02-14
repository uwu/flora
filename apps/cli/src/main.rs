mod api_types;
mod auth;
mod cli;
mod commands;
mod config;
mod files;

use clap::Parser;
use cli::{Cli, Commands};
use color_eyre::eyre::Result;
use config::CliConfig;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let mut config: CliConfig = confy::load("flora", "cli")?;
    if let Some(url) = cli.api_url.clone() {
        config.api_url = url;
    }
    let client = Client::new();

    match cli.command {
        Commands::Deploy { guild, entry, root } => {
            commands::deployments::deploy(&client, &config, guild, entry, root).await?
        }
        Commands::Get { guild } => commands::deployments::get(&client, &config, guild).await?,
        Commands::List => commands::deployments::list(&client, &config).await?,
        Commands::Health => commands::deployments::health(&client, &config).await?,
        Commands::Login { token } => {
            config.token = Some(token);
            confy::store("flora", "cli", config)?;
            println!("Saved token to config");
        }
        Commands::Kv(kv_cmd) => commands::kv::handle_kv_command(&client, &config, kv_cmd).await?,
        Commands::Logs {
            guild,
            follow,
            limit,
        } => {
            if follow {
                commands::logs::stream_logs(&client, &config, guild).await?
            } else {
                commands::logs::logs(&client, &config, guild, limit).await?
            }
        }
    }

    Ok(())
}
