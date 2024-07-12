use num_format::{Locale, ToFormattedString};

use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::utils::emojis::Grades;
use crate::utils::event_handler::Handler;

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: &Vec<&str>,
    handler: &Handler,
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
            let statistics = user.statistics.expect("User statistics not found");

            let author = CreateEmbedAuthor::new("")
                .name(format!(
                    "{}: {}pp (#{} {}#{})",
                    user.username,
                    statistics.pp,
                    statistics
                        .global_rank
                        .unwrap_or(0)
                        .to_formatted_string(&Locale::en),
                    user.country_code,
                    statistics
                        .country_rank
                        .unwrap_or(0)
                        .to_formatted_string(&Locale::en)
                ))
                .icon_url(format!(
                    "https://osu.ppy.sh/images/flags/{}.png",
                    user.country_code
                ))
                .url(format!("https://osu.ppy.sh/users${}", user.user_id));

            let peak_rank_string = match user.highest_rank {
                Some(peak_rank) => format!(
                    "#`{}` • **Achieved:** <t:{}:R>",
                    peak_rank.rank.to_formatted_string(&Locale::en),
                    peak_rank.updated_at.unix_timestamp()
                ),
                None => "#`-`".to_string(),
            };

            let fields = vec![
                ("Statistics :abacus:",
                format!(
                    "**Accuracy:**  `{:.2}` • **Level:** `{}.{:.2}%` \n**Playcount:** `{}` (`{} hrs`) \n**Peak Rank:** {}",
                    statistics.accuracy, statistics.level.current, statistics.level.progress,
                    statistics.playcount.to_formatted_string(&Locale::en), (statistics.playtime/3600).to_formatted_string(&Locale::en),
                    peak_rank_string
                ),
                false,
                ),
                ("Grades :mortar_board:", format!("{}`{}` {}`{}` {}`{}` {}`{}` {}`{}`", Grades::SSH, statistics.grade_counts.ssh, Grades::SS, statistics.grade_counts.ss, Grades::SH, statistics.grade_counts.sh, Grades::S, statistics.grade_counts.s, Grades::A, statistics.grade_counts.a), false)
            ];

            let footer = CreateEmbedFooter::new(format!(
                "Joined osu! on {}, {}, {}",
                user.join_date.year(),
                user.join_date.month(),
                user.join_date.day()
            ));

            let embed = CreateEmbed::new()
                .author(author)
                .fields(fields)
                .image(user.cover.custom_url.unwrap_or(user.cover.url))
                .thumbnail(user.avatar_url)
                .footer(footer);

            let builder = CreateMessage::new().embed(embed);

            msg.channel_id.send_message(&ctx.http, builder).await?;
        }
        Err(user_error) => {
            msg.channel_id
                .say(&ctx.http, format!("Error fetching user: {}", user_error))
                .await?;
        }
    }

    Ok(())
}
