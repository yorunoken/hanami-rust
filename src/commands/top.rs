use num_format::{Locale, ToFormattedString};

use rosu_v2::model::GameMode;
use rosu_v2::model::Grade;
use rosu_v2::prelude::*;

use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::Error;

use crate::event_handler::Handler;
use crate::utils::{emojis::Grades, helper, osu};

pub async fn execute(
    ctx: &Context,
    msg: &Message,
    args: Vec<&str>,
    handler: &Handler,
    _command_name: &str,
    command_alias: Option<&str>,
    play_index: Option<usize>,
) -> Result<(), Error> {
    let user_help = osu::get_user(&msg.author.id);

    let mode = match command_alias {
        Some("topmania") | Some("topm") | Some("tm") => GameMode::Mania,
        Some("toptaiko") | Some("topt") | Some("tt") => GameMode::Taiko,
        Some("topcatch") | Some("topc") | Some("tc") => GameMode::Catch,
        _ => user_help.as_ref().map_or(GameMode::Osu, |u| u.mode),
    };

    let (flags, args) = helper::get_flags(args.clone());

    let page = match flags.get("page") {
        Some(page) => match page.parse::<usize>() {
            Ok(ok) => ok,
            Err(_) => 0,
        },
        None => 0,
    };

    let index = match play_index {
        Some(index) => Some(index),
        None => {
            if let Some(s) = flags.get("index") {
                match s.parse::<usize>() {
                    Ok(ok) => Some(ok),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
    };

    let username = helper::get_username(args);
    let builder = match username {
        Some(username) => handle(username, mode, handler, page, index).await,
        None => {
            if let Some(user_help) = &user_help {
                handle(UserId::Id(user_help.bancho_id), mode, handler, page, index).await
            } else {
                CreateMessage::new().content("Please provide a username.")
            }
        }
    };

    msg.channel_id.send_message(&ctx.http, builder).await?;
    Ok(())
}

async fn handle(
    username: impl Into<UserId>,
    mode: GameMode,
    handler: &Handler,
    page: usize,
    index: Option<usize>,
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
        .best()
        .limit(100)
        .mode(mode)
        .await;
    let scores = match scores_result {
        Ok(ok) => ok,
        Err(error) => {
            return CreateMessage::new().content(format!("Error fetching scores: `{}`", error));
        }
    };

    // see if index is Some(...)
    match index {
        Some(index) => play_single(user, scores, mode, handler, index).await,
        None => play_list(user, scores, handler, page).await,
    }
}

async fn play_single(
    user: UserExtended,
    scores: Vec<Score>,
    mode: GameMode,
    handler: &Handler,
    index: usize,
) -> CreateMessage {
    if index > scores.len() - 1 {
        return CreateMessage::new().content("No plays found with these settings");
    }

    let score = &scores[index];

    let map = match &score.map {
        Some(s) => s,
        // I think this is basically never the case, but still good to have it
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
    let fields = create_embed_fields_single(score, mode, map, performance_options, scores.len());

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

async fn play_list(
    user: UserExtended,
    scores: Vec<Score>,
    handler: &Handler,
    page: usize,
) -> CreateMessage {
    let statistics = user.statistics.as_ref().expect("User statistics not found");
    let author = create_author_embed(&user, statistics);
    let description = create_embed_description_list(scores, handler, page).await;

    let embed = CreateEmbed::new().author(author).description(description);
    CreateMessage::new().embed(embed)
}

// Create embed fields for SINGLE plays
fn create_embed_fields_single(
    score: &Score,
    mode: GameMode,
    map: &Box<BeatmapExtended>,
    options: osu::GetPerformance,
    total_scores: usize,
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
                fc_acc = osu::calculate_accuracy(fc_stats, mode)
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
                "{ruleset_emoji} {diff_name} **+{mods}** [{stars:.2}★] Top **__#{position}__** of {total_scores}",
                ruleset_emoji = "",
                diff_name = map.version,
                mods = score.mods.to_string(),
                stars = options.diff_attrs.stars(),
                position = match score.weight {Some(s) => s.percentage.floor() as i32, None => 0},
                total_scores = total_scores
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

const ELEMENT_PER_PAGE: usize = 5;

// Create embed descriptions for LISTED plays
async fn create_embed_description_list(
    scores: Vec<Score>,
    handler: &Handler,
    page: usize,
) -> String {
    let page_start = page * ELEMENT_PER_PAGE;
    let page_end = page_start + ELEMENT_PER_PAGE;

    let scores: Vec<&Score> = scores
        .iter()
        .skip(page_start)
        .take(page_end - page_start)
        .collect();

    // Initialize description
    let mut description = "".to_string();
    for score in scores {
        let map = match &score.map {
            Some(s) => s,
            // I think this is basically never the case, but still good to have it
            None => {
                let fetched_map = match handler.osu_client.beatmap().map_id(score.map_id).await {
                    Ok(ok) => ok,
                    Err(_) => {
                        return "".to_string();
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
                    Err(_) => {
                        return "".to_string();
                    }
                };

                &Box::new(fetched_set)
            }
        };

        let performance = osu::get_performance(
            map.map_id,
            score.mode,
            score.mods.bits(),
            score.max_combo,
            &score.statistics,
        )
        .await;

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

        // let if_fc_formatted = if score.statistics.miss > 0
        //     || performance.diff_attrs.max_combo() - score.max_combo >= 20
        // {
        //     let mut fc_stats = score.statistics.clone();
        //     fc_stats.miss = 0;
        //     format!(
        //         "FC: **{fc_pp:.2}pp** for **{fc_acc:.2}%**",
        //         fc_pp = performance.fc_perf.pp(),
        //         fc_acc = osu::calculate_accuracy(fc_stats, mode)
        //     )
        // } else {
        //     "".to_string()
        // };

        let lines = format!(
            "**#{position} [{map_title} [{map_version}]]({map_link}) +{mods} {stars:.2}★**\n\
            {grade} **{pp:.2}**/{max_pp:.2}pp {score} **{accuracy:.2}%**\n\
            {{{hits}}} {combo} <t:{submitted_at}:R>\n",
            position = match score.weight {
                Some(s) => s.percentage.floor() as i32,
                None => 0,
            },
            map_title = mapset.title,
            map_version = map.version,
            map_link = map.url,
            mods = score.mods.to_string(),
            stars = performance.diff_attrs.stars(),
            // 2nd line
            grade = grade_emoji,
            pp = performance.curr_perf.pp(),
            max_pp = performance.max_perf.pp(),
            score = score.score.to_formatted_string(&Locale::en),
            accuracy = score.accuracy,
            hits = hits,
            combo = format!(
                "**{}**/{}x",
                score.max_combo,
                performance.diff_attrs.max_combo()
            ),
            submitted_at = score.ended_at.unix_timestamp()
        );

        // Push lines into description
        description.push_str(lines.as_str());
    }

    description
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
