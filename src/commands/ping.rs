use crate::Handler;
use std::time::Instant;

use serenity::builder::EditMessage;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    _args: &Vec<&str>,
    _handler: &Handler,
) -> Result<(), Error> {
    let timer_start = Instant::now();

    let content = "Pong!";
    let mut sent_message = msg.channel_id.say(&ctx.http, content).await?;

    let elapsed = (Instant::now() - timer_start).as_millis();

    let builder = EditMessage::new().content(format!("{} ({}ms)", content, elapsed));
    sent_message.edit(&ctx.http, builder).await?;

    Ok(())
}
