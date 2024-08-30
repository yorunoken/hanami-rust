use serenity::{
    all::{CommandInteraction, CreateMessage, Error},
    async_trait,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    model::channel::Message,
    prelude::*,
};

use crate::command_trait::Command;

pub struct Ping;

#[async_trait]
impl Command for Ping {
    fn name(&self) -> &'static str {
        "ping"
    }

    async fn run(
        &self,
        ctx: &Context,
        msg: &Message,
        _args: Vec<&str>,
        _command: &str,
    ) -> Result<(), Error> {
        let content = "Pong!";
        let builder = CreateMessage::new().content(content);
        msg.channel_id.send_message(&ctx.http, builder).await?;
        Ok(())
    }

    async fn run_slash(&self, ctx: &Context, command: &CommandInteraction) -> Result<(), Error> {
        let content = "Pong!";

        let builder = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content(content),
        );
        command.create_response(&ctx.http, builder).await?;
        Ok(())
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name()).description("A ping command")
    }
}
