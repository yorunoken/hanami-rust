use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use regex::Regex;

use rosu_v2::prelude::*;

use crate::options::Command;

pub struct Handler {
    pub osu_client: Osu,
    pub commands: Vec<Command>,
}

const PREFIX: &str = "'";

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
        let mut command_input = args.remove(0);

        let regex = Regex::new(r"^([a-zA-Z]+)(\d*)$").unwrap();
        let captures = regex.captures(&command_input);

        // Initialize command variables
        let mut index: usize = 0;

        if let Some(caps) = captures {
            command_input = caps.get(1).map_or("", |m| m.as_str());
            if let Some(matched_index) = caps.get(2) {
                index = matched_index.as_str().parse().unwrap_or(1) - 1;
            }
        }

        for command in &self.commands {
            if command.name == command_input || command.aliases.contains(&command_input) {
                let matched_alias = if command.name == command_input {
                    None
                } else {
                    Some(command_input)
                };

                // Start typing
                msg.channel_id.start_typing(&ctx.http);

                // Execute command
                if let Err(reason) = (command.exec)(
                    &ctx,
                    &msg,
                    args,
                    &self,
                    command.name,
                    matched_alias,
                    Some(index),
                    Some(0 as usize),
                )
                .await
                {
                    println!(
                        "There was an error while handling command {}: {:#?}",
                        command.name, reason
                    )
                }

                return;
            }
        }
    }
}
