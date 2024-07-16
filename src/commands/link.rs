use serenity::all::CreateMessage;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use rusqlite::Connection;

use crate::event_handler::Handler;

struct User {
    bancho_id: Option<u32>,
}

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    handler: &Handler,
    _command_name: &str,
    _command_alias: Option<&str>,
) -> Result<(), Error> {
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
            let builder =
                CreateMessage::new().content(format!("Error fetching user: `{}`", user_error));
            msg.channel_id.send_message(&ctx.http, builder).await?;
            return Ok(());
        }
    }

    if let Some(d) = user.bancho_id {
        let builder = CreateMessage::new().content(format!("You're already linked to user id: {d}.\n If this command was intentional, you should unlink and re-link yourself."));
        msg.channel_id.send_message(&ctx.http, builder).await?;

        return Ok(());
    }

    if let Some(bancho_id) = bancho_id {
        let query = "INSERT INTO `users` (id, bancho_id)
        VALUES (?1, ?2)
        ON CONFLICT(id) DO UPDATE SET bancho_id = excluded.bancho_id;";
        connection
            .execute(query, [&msg.author.id.to_string(), &bancho_id.to_string()])
            .unwrap();

        let builder =
            CreateMessage::new().content(format!("Linked your Discord account to {}", bancho_id));
        msg.channel_id.send_message(&ctx.http, builder).await?;

        return Ok(());
    }

    let builder = CreateMessage::new().content("how tf are you here\nreport it to @yorunoken pls");
    msg.channel_id.send_message(&ctx.http, builder).await?;

    Ok(())
}
