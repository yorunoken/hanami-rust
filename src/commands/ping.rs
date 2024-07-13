use crate::utils::event_handler::Handler;
use std::time::Instant;

use serenity::builder::EditMessage;
use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

pub fn execute<'a>(
    ctx: &'a Context,
    msg: &'a Message,
    _args: Vec<&'a str>,
    _handler: &'a Handler,
    _command_name: &'a str,
    _command_alias: Option<&'a str>,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        let timer_start = Instant::now();

        let content = "Pong!";
        let mut sent_message = msg.channel_id.say(&ctx.http, content).await?;

        let elapsed = (Instant::now() - timer_start).as_millis();

        let builder = EditMessage::new().content(format!("{} ({}ms)", content, elapsed));
        sent_message.edit(&ctx.http, builder).await?;

        Ok(())
    })
}
