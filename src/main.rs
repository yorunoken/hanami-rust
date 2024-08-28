use dotenvy::dotenv;
use serenity::prelude::*;
use std::env;

mod command_trait;
mod commands;
mod event_handler;
mod options;
mod services;
mod utils;

#[tokio::main]
async fn main() {
    // Load the environment variables
    dotenv().ok();

    let osu_client = services::osu::get_client().await;

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN to be defined in environment.");

    let database_uri =
        env::var("DATABASE_URI").expect("Expected DISCORD_TOKEN to be defined in environment.");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Get commands
    let commands = options::get_commands();

    // Build the Discord client, and pass in our event handler
    let mut client = Client::builder(discord_token, intents)
        .event_handler(event_handler::Handler { commands })
        .await
        .expect("Error creating client.");

    let database = services::database::connect(database_uri).await;

    {
        let mut data = client.data.write().await;

        // Insert database into Discord client
        data.insert::<services::database::Database>(database);
        // Insert osu! client into Discord client
        data.insert::<services::osu::OsuClient>(osu_client);
    }

    // Run the Discord client (runs the ready function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
