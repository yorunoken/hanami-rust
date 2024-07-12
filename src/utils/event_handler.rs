use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use rosu_v2::prelude::*;

use crate::commands;

pub struct Handler {
    pub osu_client: Osu,
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
                if let Err(reason) = commands::ping::execute(&ctx, &msg, &args, self).await {
                    println!("There was an error sending message: {:#?}", reason);
                };
            }

            "osu" => {
                if let Err(reason) = commands::profile::execute(&ctx, &msg, &args, self).await {
                    println!("There was an error sending message: {:#?}", reason);
                };
            }

            _ => {}
        }
    }
}
