use std::env;

use dotenv::dotenv;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

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

        // Get the command name by removing the first arg of the `args` array
        let command_name = args.remove(0);

        if command_name.to_lowercase() == "ping" {
            if let Err(reason) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("There was an error sending message: {:#?}", reason);
            };
        }
    }
}

#[tokio::main]
async fn main() {
    // Load the environment variables
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected token to be defined in environment.");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Build the client, and pass in our event handler
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client.");

    // Run the client (runs the `ready` function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
