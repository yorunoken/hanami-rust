use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::Handler;

pub async fn osu(
    ctx: &Context,
    msg: &Message,
    handler: &Handler,
    args: Vec<&str>,
) -> Result<(), Error> {
    let username = args.join(" ");
    if username.is_empty() {
        msg.channel_id
            .say(&ctx.http, "Please provide a username.")
            .await?;
        return Ok(());
    }

    match handler.osu_client.user(username).await {
        Ok(user) => {
            let response = format!("Username: {}", user.username);
            msg.channel_id.say(&ctx.http, response).await?;
        }
        Err(user_error) => {
            msg.channel_id
                .say(&ctx.http, format!("Error fetching user: {}", user_error))
                .await?;
        }
    }

    Ok(())
}
