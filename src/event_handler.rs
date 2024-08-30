use crate::command_trait::Command;
use serenity::{
    async_trait,
    model::{application::Command as ApplicationCommand, prelude::*},
    prelude::*,
};

pub struct Handler {
    pub commands: Vec<Box<dyn Command + Send + Sync>>,
}

const PREFIX: &str = "'";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot || !msg.content.starts_with(PREFIX) {
            return;
        }

        let mut args: Vec<&str> = msg.content[1..].split_whitespace().collect();
        let command_name = args.remove(0).to_lowercase();

        for command in &self.commands {
            if command.name() == command_name || command.aliases().contains(&command_name.as_str())
            {
                if let Err(why) = command.run(&ctx, &msg, args, command_name.as_str()).await {
                    println!("Error executing command: {:?}", why);
                }
                return;
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            for cmd in &self.commands {
                if cmd.name() == command.data.name.as_str() {
                    if let Err(why) = cmd.run_slash(&ctx, &command).await {
                        println!("Error executing slash command: {:?}", why);
                    }
                    return;
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let commands = self
            .commands
            .iter()
            .map(|c| c.register())
            .collect::<Vec<_>>();

        ApplicationCommand::set_global_commands(&ctx.http, commands)
            .await
            .expect("Failed to set global application commands");
    }
}
