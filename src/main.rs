use std::env;

use dotenv::dotenv;

use rosu_v2::prelude::*;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands {
    pub mod osu;
    pub mod ping;
}

struct Handler {
    osu_client: Osu,
}

const PREFIX: &str = "!";

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, bot: Ready) {
        println!("Bot has started as {}", bot.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Make sure we're dealing with humans :)
        if msg.author.bot || msg.content.len() == 0 {
            return;
        }

        // Message doesn't start with the prefix, meaning it's not a command. So we return
        if !msg.content.starts_with(PREFIX) {
            return;
        }

        // Get the arguments
        let mut args: Vec<&str> = msg
            .content
            .strip_prefix(PREFIX)
            .unwrap()
            .split_whitespace()
            .collect();

        // Get the command name by removing the first arg of the args array
        let command = args.remove(0);

        match command.to_lowercase().as_str() {
            "ping" => {
                if let Err(reason) = commands::ping::ping(&ctx, &msg).await {
                    println!("There was an error sending message: {:#?}", reason);
                };
            }

            "osu" => {
                if let Err(reason) = commands::osu::osu(&ctx, &msg, self, args).await {
                    println!("There was an error sending message: {:#?}", reason);
                };
            }

            _ => {}
        }
    }
}

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
        .event_handler(Handler { osu_client })
        .await
        .expect("Error creating client.");

    // Run the Discord client (runs the ready function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
