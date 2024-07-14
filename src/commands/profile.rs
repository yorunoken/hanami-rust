use chrono::{Datelike, Utc};
use num_format::{Locale, ToFormattedString};

use rosu_v2::error::OsuError;
use rosu_v2::model::GameMode;

use rosu_v2::prelude::{UserExtended, UserId, UserStatistics};
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::utils::{emojis::Grades, event_handler::Handler, osu_helper::get_user};

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    handler: &Handler,
    _command_name: &str,
    command_alias: Option<&str>,
) -> Result<(), Error> {
    let user_help = get_user(&msg.author.id);
    println!("{:#?}", user_help);

    let mode = match command_alias {
        Some("mania") => GameMode::Mania,
        Some("taiko") => GameMode::Taiko,
        Some("ctb") => GameMode::Catch,
        _ => user_help.as_ref().map_or(GameMode::Osu, |u| u.mode),
    };

    let mut osu_user_result: Option<Result<UserExtended, OsuError>> = None;

    let mut username = args.join(" ");
    if username.is_empty() {
        if let Some(user_help) = &user_help {
            return fetch_and_send_user_data(
                ctx,
                msg,
                UserId::Id(user_help.bancho_id),
                mode,
                handler,
            )
            .await;
        } else {
            msg.channel_id
                .say(&ctx.http, "Please provide a username.")
                .await?;
            return Ok(());
        }
    }

    fetch_and_send_user_data(ctx, msg, UserId::Name(username.into()), mode, handler).await
}

async fn fetch_and_send_user_data(
    ctx: &Context,
    msg: &Message,
    username: impl Into<UserId>,
    mode: GameMode,
    handler: &Handler,
) -> Result<(), Error> {
    let osu_user_result = handler.osu_client.user(username).mode(mode).await;

    match osu_user_result {
        Ok(user) => {
            let statistics = user.statistics.clone().expect("User statistics not found");

            let author = create_author_embed(&user, &statistics);
            let fields = create_embed_fields(&statistics);
            let footer = create_footer(&user);

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
                .say(&ctx.http, format!("Error fetching user: `{}`", user_error))
                .await?;
        }
    }

    Ok(())
}

fn create_author_embed(user: &UserExtended, statistics: &UserStatistics) -> CreateEmbedAuthor {
    CreateEmbedAuthor::new("")
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
        .url(format!("https://osu.ppy.sh/users/{}", user.user_id))
}

fn create_embed_fields(statistics: &UserStatistics) -> Vec<(&str, String, bool)> {
    vec![
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
                peak_rank = match &statistics.highest_rank {
                    Some(peak_rank) => format!(
                        "#`{}` • **Achieved:** <t:{}:R>",
                        peak_rank.rank.to_formatted_string(&Locale::en),
                        peak_rank.updated_at.unix_timestamp()
                    ),
                    None => "#`-`".to_string(),
                },
                followers = user
                    .follower_count
                    .unwrap_or(0)
                    .to_formatted_string(&Locale::en),
                max_combo = statistics.max_combo.to_formatted_string(&Locale::en),
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
    ]
}

fn create_footer(user: &UserExtended) -> CreateEmbedFooter {
    CreateEmbedFooter::new(format!(
        "Joined osu! on {}, {} {}, {}:{} ({} yrs ago)",
        user.join_date.year(),
        user.join_date.month(),
        user.join_date.day(),
        user.join_date.hour(),
        user.join_date.minute(),
        Utc::now().year() - user.join_date.year()
    ))
}
