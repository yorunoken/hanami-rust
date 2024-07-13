use serenity::futures::future::BoxFuture;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use rusqlite::{Connection, Result};

use crate::utils::event_handler::Handler;

struct User {
    bancho_id: Option<String>,
}

pub fn execute<'a>(
    ctx: &'a Context,
    msg: &'a Message,
    args: Vec<&'a str>,
    handler: &'a Handler,
    _command_name: &'a str,
    _command_alias: Option<&'a str>,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        // Connect to db
        let connection = Connection::open("./data.db").unwrap();

        let username = args.join(" ");

        // User related
        let mut bancho_id: Option<u32> = None;
        let mut user = User { bancho_id: None };

        match handler.osu_client.user(username).await {
            Ok(osu_user) => {
                bancho_id = Some(osu_user.user_id);
                let query = "SELECT * FROM users WHERE id = ?1";
                let mut stmt = connection.prepare(query).unwrap();
                let mut rows = stmt.query([&msg.author.id.to_string()]).unwrap();

                while let Some(row) = rows.next().unwrap() {
                    user = User {
                        bancho_id: row.get("bancho_id").ok(),
                    };
                }
            }
            Err(user_error) => {
                msg.channel_id
                    .say(&ctx.http, format!("Error fetching user: `{}`", user_error))
                    .await?;
                return Ok(());
            }
        }

        if let Some(d) = user.bancho_id {
            msg.channel_id
                .say(&ctx.http, format!("You're already linked to user id: {d}.\n If this command was intentional, you should unlink and re-link yourself."))
                .await?;

            return Ok(());
        }

        if let Some(bancho_id) = bancho_id {
            let query = "INSERT INTO `users` (id, bancho_id)
        VALUES (?1, ?2)
        ON CONFLICT(id) DO UPDATE SET bancho_id = excluded.bancho_id;";
            connection
                .execute(query, [&msg.author.id.to_string(), &bancho_id.to_string()])
                .unwrap();

            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Linked your Discord account to {}", bancho_id),
                )
                .await?;

            return Ok(());
        }

        msg.channel_id
            .say(
                &ctx.http,
                "how tf are you here\nreport it to @yorunoken pls",
            )
            .await?;

        Ok(())
    })
}
