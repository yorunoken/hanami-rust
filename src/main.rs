use std::env;

use dotenv::dotenv;

use rosu_v2::prelude::*;

use serenity::prelude::*;

mod commands {
    pub mod ping;
    pub mod profile;
}

mod utils;

#[tokio::main]
async fn main() {
    // Load the environment variables
    dotenv().ok();

    // Get and parse osu! client information
    let osu_client_id: u64 = env::var("OSU_CLIENT_ID")
        .expect("Expected OSU_CLIENT_ID to be defined in environment.")
        .parse()
        .expect("OSU_CLIENT_ID is not a number!");

    let osu_client_secret = env::var("OSU_CLIENT_SECRET")
        .expect("Expected OSU_CLIENT_SECRET to be defined in environment.");

    // Build the osu! client
    let osu_client = Osu::new(osu_client_id, osu_client_secret).await.unwrap();

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN to be defined in environment.");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Build the Discord client, and pass in our event handler
    let mut client = Client::builder(discord_token, intents)
        .event_handler(utils::event_handler::Handler { osu_client })
        .await
        .expect("Error creating client.");

    // Run the Discord client (runs the ready function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
