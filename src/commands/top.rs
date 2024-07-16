use chrono::{Datelike, Utc};
use num_format::{Locale, ToFormattedString};

use rosu_v2::model::GameMode;
use rosu_v2::model::Grade;
use rosu_v2::prelude::*;

use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::event_handler::Handler;
use crate::utils::{emojis::Grades, osu};

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    handler: &Handler,
    _command_name: &str,
    command_alias: Option<&str>,
    play_index: Option<usize>,
    _play_page: Option<usize>,
) -> Result<(), Error> {
    let user_help = osu::get_user(&msg.author.id);

    let mode = match command_alias {
        Some("topmania") | Some("topm") | Some("tm") => GameMode::Mania,
        Some("toptaiko") | Some("topt") | Some("tt") => GameMode::Taiko,
        Some("topcatch") | Some("topc") | Some("tc") => GameMode::Catch,
        _ => user_help.as_ref().map_or(GameMode::Osu, |u| u.mode),
    };

    let page = 0;

    let username = args.join(" ");
    if username.is_empty() {
        if let Some(user_help) = &user_help {
            let builder = handle(UserId::Id(user_help.bancho_id), mode, handler, page).await;

            msg.channel_id.send_message(&ctx.http, builder).await?;
            return Ok(());
        } else {
            let builder = CreateMessage::new().content("Please provide a username.");
            msg.channel_id.send_message(&ctx.http, builder).await?;

            return Ok(());
        }
    }

    let builder = handle(UserId::Name(username.into()), mode, handler, page).await;

    msg.channel_id.send_message(&ctx.http, builder).await?;
    Ok(())
}

async fn handle(
    username: impl Into<UserId>,
    mode: GameMode,
    handler: &Handler,
    page: usize,
) -> CreateMessage {
    let user_result = handler.osu_client.user(username).mode(mode).await;
    let user = match user_result {
        Ok(ok) => ok,
        Err(reason) => {
            return CreateMessage::new().content(format!("Error fetching user: `{}`", reason));
        }
    };

    let scores_result = handler
        .osu_client
        .user_scores(user.user_id)
        .best()
        .limit(100)
        .mode(mode)
        .await;
    let scores = match scores_result {
        Ok(ok) => ok,
        Err(error) => {
            return CreateMessage::new().content(format!("Error fetching scores: `{}`", error));
        }
    };

    CreateMessage::new().content(format!("Error fetching user: ``"))
}

// Implement other shit here, gn
