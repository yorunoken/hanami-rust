use chrono::{Datelike, Utc};
use num_format::{Locale, ToFormattedString};

use rosu_v2::model::GameMode;
use rosu_v2::prelude::{UserExtended, UserId, UserStatistics};

use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::utils::{emojis::Grades, event_handler::Handler, osu::get_user};

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    handler: &Handler,
    _command_name: &str,
    command_alias: Option<&str>,
) {
    let user_help = get_user(&msg.author.id);

    let mode = match command_alias {
        Some("mania") => GameMode::Mania,
        Some("taiko") => GameMode::Taiko,
        Some("catch") => GameMode::Catch,
        _ => user_help.as_ref().map_or(GameMode::Osu, |u| u.mode),
    };

    let username = args.join(" ");
    if username.is_empty() {
        if let Some(user_help) = &user_help {
            let builder = create_message(UserId::Id(user_help.bancho_id), mode, handler).await;
            msg.channel_id.send_message(&ctx.http, builder).await;
            return;
        } else {
            let builder = CreateMessage::new().content("Please provide a username.");
            msg.channel_id.send_message(&ctx.http, builder).await;

            return;
        }
    }

    let builder = create_message(UserId::Name(username.into()), mode, handler).await;
    msg.channel_id.send_message(&ctx.http, builder).await;
}

async fn create_message(
    username: impl Into<UserId>,
    mode: GameMode,
    handler: &Handler,
) -> CreateMessage {
    let user_result = handler.osu_client.user(username).mode(mode).await;
    let user = match user_result {
        Ok(ok) => ok,
        Err(user_error) => {
            return CreateMessage::new().content(format!("Error fetching user: `{}`", user_error));
        }
    };

    let statistics = user.statistics.as_ref().expect("User statistics not found");

    let author = create_author_embed(&user, statistics);
    let fields = create_embed_fields(&user, statistics);
    let footer = create_footer(&user);

    let embed = CreateEmbed::new()
        .author(author)
        .fields(fields)
        .image(user.cover.custom_url.unwrap_or(user.cover.url))
        .thumbnail(user.avatar_url)
        .footer(footer);

    CreateMessage::new().embed(embed)
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

fn create_embed_fields(
    user: &UserExtended,
    statistics: &UserStatistics,
) -> Vec<(String, String, bool)> {
    vec![
        (
            "Statistics :abacus:".to_string(),
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
                peak_rank = match &user.highest_rank {
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
            "Grades :mortar_board:".to_string(),
            format!(
                "{}`{}` {}`{}` {}`{}` {}`{}` {}`{}`",
                Grades::XH,
                statistics.grade_counts.ssh,
                Grades::X,
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
