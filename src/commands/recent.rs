use chrono::{Datelike, Utc};
use num_format::{Locale, ToFormattedString};

use rosu_v2::model::GameMode;
use rosu_v2::model::Grade;
use rosu_v2::prelude::*;

use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::utils::{emojis::Grades, event_handler::Handler, osu};

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    handler: &Handler,
    _command_name: &str,
    command_alias: Option<&str>,
    play_index: Option<usize>,
    _play_page: Option<usize>,
) -> () {
    let user_help = osu::get_user(&msg.author.id);

    let mode = match command_alias {
        Some(alias) if alias.starts_with("recentmania") || alias.starts_with("rm") => {
            GameMode::Mania
        }
        Some(alias) if alias.starts_with("recenttaiko") || alias.starts_with("rt") => {
            GameMode::Taiko
        }
        Some(alias) if alias.starts_with("recentcatch") || alias.starts_with("rc") => {
            GameMode::Catch
        }
        _ => user_help.as_ref().map_or(GameMode::Osu, |u| u.mode),
    };

    let include_fails = match command_alias {
        Some(alias) if alias.ends_with("pass") || alias.ends_with('p') => false,
        _ => true,
    };

    let index = match play_index {
        Some(index) => index,
        None => 0,
    };

    let username = args.join(" ");
    if username.is_empty() {
        if let Some(user_help) = &user_help {
            let builder = create_message(
                UserId::Id(user_help.bancho_id),
                mode,
                handler,
                index,
                include_fails,
            )
            .await;

            msg.channel_id.send_message(&ctx.http, builder);
            return;
        } else {
            let builder = CreateMessage::new().content("Please provide a username.");
            msg.channel_id.send_message(&ctx.http, builder);

            return;
        }
    }

    let builder = create_message(
        UserId::Name(username.into()),
        mode,
        handler,
        index,
        include_fails,
    )
    .await;

    msg.channel_id.send_message(&ctx.http, builder).await;
    return;
}

async fn create_message(
    username: impl Into<UserId>,
    mode: GameMode,
    handler: &Handler,
    index: usize,
    include_fails: bool,
) -> CreateMessage {
    let user_result = handler.osu_client.user(username).mode(mode).await;
    let user = match user_result {
        Ok(ok) => ok,
        Err(reason) => {
            return CreateMessage::new().content(format!("Error fetching user: `{}`", reason));
        }
    };

    let scores_result = handler
        .osu_client
        .user_scores(user.user_id)
        .recent()
        .include_fails(include_fails)
        .mode(mode)
        .await;
    let scores = match scores_result {
        Ok(ok) => ok,
        Err(error) => {
            return CreateMessage::new().content(format!("Error fetching scores: `{}`", error));
        }
    };

    if index > scores.len() - 1 {
        return CreateMessage::new().content("No plays found with these settings");
    }

    let score = &scores[index];

    let map = match &score.map {
        Some(s) => s,
        None => {
            let fetched_map = match handler.osu_client.beatmap().map_id(score.map_id).await {
                Ok(ok) => ok,
                Err(reason) => {
                    return CreateMessage::new()
                        .content(format!("Error fetching scores: `{}`", reason));
                }
            };

            &Box::new(fetched_map)
        }
    };

    let mapset = match &map.mapset {
        Some(s) => s,
        None => {
            let fetched_set = match handler.osu_client.beatmapset(map.mapset_id).await {
                Ok(ok) => ok,
                Err(reason) => {
                    return CreateMessage::new()
                        .content(format!("Error fetching scores: `{}`", reason));
                }
            };

            &Box::new(fetched_set)
        }
    };

    let performance_options = osu::get_performance(
        map.map_id,
        score.mode,
        score.mods.bits(),
        score.max_combo,
        &score.statistics,
    )
    .await;

    let statistics = user.statistics.as_ref().expect("User statistics not found");

    let author = create_author_embed(&user, statistics);
    let fields = create_embed_fields(score, mode, map, performance_options);

    let embed = CreateEmbed::new()
        .author(author)
        .fields(fields)
        .image(format!(
            "https://assets.ppy.sh/beatmaps/{}/covers/cover.jpg",
            map.mapset_id
        ))
        .title(format!("{} - {}", mapset.artist, mapset.title))
        .url(&map.url)
        .thumbnail(user.avatar_url);

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
    score: &Score,
    mode: GameMode,
    map: &Box<BeatmapExtended>,
    options: osu::GetPerformance,
) -> Vec<(String, String, bool)> {
    let grade_emoji = match score.grade {
        Grade::XH => Grades::XH,
        Grade::X => Grades::X,
        Grade::SH => Grades::SH,
        Grade::S => Grades::S,
        Grade::A => Grades::A,
        Grade::B => Grades::B,
        Grade::C => Grades::C,
        Grade::D => Grades::D,
        Grade::F => Grades::F,
    };

    let percentage_passed = if !score.passed {
        let total_hits = score.total_hits() as f64;
        let count_objects = map.count_objects() as f64;
        format!("**@{:.2}%**", (total_hits / count_objects) * 100.0)
    } else {
        "".to_string()
    };

    let if_fc_formatted =
        if score.statistics.miss > 0 || options.diff_attrs.max_combo() - score.max_combo >= 20 {
            let mut fc_stats = score.statistics.clone();
            fc_stats.miss = 0;
            format!(
                "FC: **{fc_pp:.2}pp** for **{fc_acc:.2}%**",
                fc_pp = options.fc_perf.pp(),
                fc_acc = osu::accuracy(fc_stats, mode)
            )
        } else {
            "".to_string()
        };

    let hits = match score.mode {
        GameMode::Osu => format!(
            "{}/{}/{}/{}",
            score.statistics.great,
            score.statistics.ok,
            score.statistics.meh,
            score.statistics.miss
        ),
        GameMode::Mania => format!(
            "{}/{}/{}/{}/{}/{}",
            score.statistics.perfect,
            score.statistics.great,
            score.statistics.good,
            score.statistics.ok,
            score.statistics.meh,
            score.statistics.miss
        ),
        GameMode::Taiko => format!(
            "{}/{}/{}",
            score.statistics.great, score.statistics.ok, score.statistics.miss
        ),
        GameMode::Catch => format!(
            "{}/{}/{}/{}",
            score.statistics.great,
            score.statistics.large_tick_hit,
            score.statistics.large_tick_miss,
            score.statistics.miss
        ),
    };

    // name, value, inline
    vec![
        (
            format!(
                "{ruleset_emoji} {diff_name} **+{mods}** [{stars:.2}★]",
                ruleset_emoji = "",
                diff_name = map.version,
                mods = score.mods.to_string(),
                stars = options.diff_attrs.stars()
            ),
            format!(
                "{grade_emoji} {percentage_passed} {total_score} **{accuracy:.2}%** <t:{submitted_at}:R>\n\
            **{pp:.2}**/{max_pp:.2} [{combo}] {{{hits}}}\n\
            {if_fc_formatted}",
                grade_emoji = grade_emoji,
                percentage_passed = percentage_passed,
                total_score = score.score.to_formatted_string(&Locale::en),
                accuracy = score.accuracy,
                submitted_at = score.ended_at.unix_timestamp(),
                pp = options.curr_perf.pp(),
                max_pp = options.max_perf.pp(),
                combo = format!("**{}**/{}x", score.max_combo, options.diff_attrs.max_combo()),
                hits = hits
            ),
            false,
        ),
        (
            "Beatmap Info:".to_string(),
            format!(
                "**BPM:** `{bpm:.0}` **Length:** {minutes}:{seconds:02}\n\
            **AR:** `{ar:.1}` **OD:** `{od:.1}` **CS:** `{cs:.1}` **HP:** `{hp:.1}`",
                bpm = options.map.bpm(),
                minutes = map.seconds_drain / 60,
                seconds = map.seconds_drain % 60,
                ar = options.map.ar,
                od = options.map.od,
                cs = options.map.cs,
                hp = options.map.hp,
            ),
            false,
        ),
    ]
}
