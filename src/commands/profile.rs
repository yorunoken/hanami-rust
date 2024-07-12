use chrono::{Datelike, Utc};
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
    // Start typing
    msg.channel_id.start_typing(&ctx.http);

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

            // Build author embed
            let author = CreateEmbedAuthor::new("")
                .name(format!(
                    "{username}: {pp}pp (#{global_rank} {country_code}#{country_rank})",
                    username = user.username,
                    pp = statistics.pp,
                    global_rank = statistics
                        .global_rank
                        .unwrap_or(0)
                        .to_formatted_string(&Locale::en),
                    country_code = user.country_code,
                    country_rank = statistics
                        .country_rank
                        .unwrap_or(0)
                        .to_formatted_string(&Locale::en)
                ))
                .icon_url(format!(
                    "https://osu.ppy.sh/images/flags/{}.png",
                    user.country_code
                ))
                .url(format!("https://osu.ppy.sh/users/{}", user.user_id));

            let peak_rank_string = match user.highest_rank {
                Some(peak_rank) => format!(
                    "#`{}` • **Achieved:** <t:{}:R>",
                    peak_rank.rank.to_formatted_string(&Locale::en),
                    peak_rank.updated_at.unix_timestamp()
                ),
                None => "#`-`".to_string(),
            };

            let fields = vec![
                (
                    "Statistics :abacus:",
                    format!(
                        "**Accuracy:**  `{accuracy:.2}` • **Level:** `{level}.{progress:02}` \n\
                        **Playcount:** `{playcount}` (`{playtime} hrs`) \n\
                        **Peak Rank:** {peak_rank} \n\
                        **Followers:** `{followers}` • **Max Combo:** `{max_combo}` \n\
                        **Recommended Star Rating:** `{star_rating:.2}`★",
                        accuracy = statistics.accuracy,
                        level = statistics.level.current,
                        progress = statistics.level.progress,
                        playcount = statistics.playcount.to_formatted_string(&Locale::en),
                        playtime = (statistics.playtime / 3600).to_formatted_string(&Locale::en),
                        peak_rank = peak_rank_string,
                        followers = user
                            .follower_count
                            .unwrap_or(0)
                            .to_formatted_string(&Locale::en),
                        max_combo = statistics.max_combo.to_formatted_string(&Locale::en),
                        // Gets recommended star rating
                        // https://www.reddit.com/r/osugame/comments/rj6pw4/comment/hp1jerz
                        star_rating = statistics.pp.powf(0.4) * 0.195
                    ),
                    false,
                ),
                (
                    "Grades :mortar_board:",
                    format!(
                        "{}`{}` {}`{}` {}`{}` {}`{}` {}`{}`",
                        Grades::SSH,
                        statistics.grade_counts.ssh,
                        Grades::SS,
                        statistics.grade_counts.ss,
                        Grades::SH,
                        statistics.grade_counts.sh,
                        Grades::S,
                        statistics.grade_counts.s,
                        Grades::A,
                        statistics.grade_counts.a
                    ),
                    false,
                ),
            ];

            let footer = CreateEmbedFooter::new(format!(
                "Joined osu! on {}, {} {}, {}:{} ({} yrs ago)",
                user.join_date.year(),
                user.join_date.month(),
                user.join_date.day(),
                user.join_date.hour(),
                user.join_date.minute(),
                Utc::now().year() - user.join_date.year()
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
