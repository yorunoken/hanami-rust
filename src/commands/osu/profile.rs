use rosu_v2::model::GameMode;
use serenity::{
    all::{CommandInteraction, CreateMessage, Error},
    async_trait,
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    model::channel::Message,
    prelude::*,
};

use crate::command_trait::Command;

pub struct Profile;

#[async_trait]
impl Command for Profile {
    fn name(&self) -> &'static str {
        "profile"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec![]
    }

    async fn run(
        &self,
        ctx: &Context,
        msg: &Message,
        args: Vec<&str>,
        command: &str,
    ) -> Result<(), Error> {
        let mode = match ProfileT::str_to_mode(command) {
            Ok(mode) => mode,
            Err(err) => {
                let builder = CreateMessage::new().content(err);
                msg.channel_id.send_message(&ctx.http, builder).await?;
                return Ok(());
            }
        };

        let option = ProfileT::args(mode, args);

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
        CreateCommand::new(self.name()).description("Profile command")
    }
}

async fn profile(ctx: &Context, msg: &Message, mode: GameMode) {}

pub struct ProfileT {
    name: Option<String>,
    mode: GameMode,
}

impl ProfileT {
    fn args(mode: GameMode, args: Vec<&str>) -> Result<Self, String> {
        let mut name = None;

        if true {
            name = Some(args.join(""));
        }

        Ok(Self { mode, name })
    }

    fn str_to_mode(mode: &str) -> Result<GameMode, String> {
        match mode {
            "osu" | "o" => Ok(GameMode::Osu),
            "taiko" | "t" => Ok(GameMode::Taiko),
            "mania" | "m" => Ok(GameMode::Mania),
            "ctb" | "catch" | "fruits" => Ok(GameMode::Catch),

            // Placeholder, will make this one default to the preferred mode of the user soon.
            "profile" => Ok(GameMode::Osu),
            _ => {
                let content = "Couldn't parse the mode.";

                return Err(content.into());
            }
        }
    }
}
