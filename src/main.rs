use std::env;

use dotenv::dotenv;

use rosu_v2::prelude::*;

use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

mod commands {
    pub mod link;
    pub mod ping;
    pub mod profile;
}

use crate::commands::link;
use crate::commands::ping;
use crate::commands::profile;

mod utils;

type CommandFn = for<'a> fn(
    &'a Context,                       // Command context, `ctx`
    &'a Message,                       // Message variable, `msg`
    Vec<&'a str>,                      // Aliases, `aliases`
    &'a utils::event_handler::Handler, // The handler, `handler`
    &'a str,                           // The command's name, `command_name`
    Option<&'a str>,                   // The command's alias (if it was passed)
) -> BoxFuture<'a, Result<(), Error>>;

struct Command {
    name: &'static str,
    aliases: Vec<&'static str>,
    exec: CommandFn,
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

    // Define commands
    let commands = vec![
        Command {
            name: "ping",
            aliases: vec!["ping"],
            exec: ping::execute,
        },
        Command {
            name: "profile",
            aliases: vec!["osu", "mania", "taiko", "ctb"],
            exec: profile::execute,
        },
        Command {
            name: "link",
            aliases: vec!["link"],
            exec: link::execute,
        },
    ];

    // Build the Discord client, and pass in our event handler
    let mut client = Client::builder(discord_token, intents)
        .event_handler(utils::event_handler::Handler {
            osu_client,
            commands,
        })
        .await
        .expect("Error creating client.");

    // Run the Discord client (runs the ready function)
    if let Err(reason) = client.start().await {
        println!("Error starting client: {:?}", reason);
    }
}
