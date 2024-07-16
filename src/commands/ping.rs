use crate::utils::event_handler::Handler;
use std::time::Instant;

use serenity::all::CreateMessage;
use serenity::builder::EditMessage;
use serenity::model::channel::Message;
use serenity::prelude::*;

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    _args: Vec<&str>,
    _handler: &Handler,
    _command_name: &str,
    _command_alias: Option<&str>,
) {
    let timer_start = Instant::now();

    let content = "Pong!";
    let builder = CreateMessage::new().content(content);
    let mut sent_message = msg
        .channel_id
        .send_message(&ctx.http, builder)
        .await
        .expect("Failed to send message in ping.rs");

    let elapsed = (Instant::now() - timer_start).as_millis();

    let builder = EditMessage::new().content(format!("{} ({}ms)", content, elapsed));
    sent_message.edit(&ctx.http, builder);
}
