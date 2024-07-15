use rosu_pp::model::mode::GameMode as RosuGameMode;
use rosu_v2::model::GameMode;
use rosu_v2::prelude::*;

use tokio::task;

use rosu_pp::{
    any::{DifficultyAttributes, PerformanceAttributes},
    Beatmap, Difficulty, Performance,
};

use rusqlite::Connection;

use serenity::all::UserId;

struct DatabaseUser {
    id: Option<String>,
    bancho_id: Option<u32>,
    mode: Option<String>,
}

pub struct User {
    pub bancho_id: u32,
    pub mode: GameMode,
}

pub struct GetPerformance {
    pub diff_attrs: DifficultyAttributes,
    pub max_perf: PerformanceAttributes,
    pub curr_perf: PerformanceAttributes,
    pub fc_perf: PerformanceAttributes,
    pub map: Beatmap,
}

pub fn get_user(discord_id: &UserId) -> Option<User> {
    // Connect to db
    let connection = Connection::open("./data.db").unwrap();

    let query = "SELECT * FROM users WHERE id = ?1";
    let mut stmt = connection.prepare(query).unwrap();
    let mut rows = stmt.query([&discord_id.to_string()]).unwrap();

    let db_user = match rows.next().unwrap() {
        Some(row) => DatabaseUser {
            id: row.get("id").ok(),
            bancho_id: row.get("bancho_id").ok(),
            mode: row.get("mode").ok(),
        },
        None => return None,
    };

    let bancho_id = match db_user.bancho_id {
        Some(id) => id,
        None => return None,
    };

    let mode: GameMode = match db_user.mode {
        Some(mode) => match mode.as_str() {
            "mania" => GameMode::Mania,
            "taiko" => GameMode::Taiko,
            "ctb" => GameMode::Catch,
            _ => GameMode::Osu,
        },
        None => GameMode::Osu,
    };

    Some(User { bancho_id, mode })
}

pub async fn download_beatmap_and_give_data(beatmap_id: u32) -> Result<String, reqwest::Error> {
    let url = format!("https://osu.ppy.sh/osu/{}", beatmap_id);

    // Download the contents
    let response_result = reqwest::get(&url).await?;
    let contents = response_result.text().await?;
    let contents_copy = contents.clone();

    // No fucking clue what's happening
    task::spawn_blocking(move || {
        let connection = Connection::open("./data.db").unwrap();
        let query = "INSERT OR REPLACE INTO `maps` (id, data) VALUES (?1, ?2);";
        connection
            .execute(query, [&beatmap_id.to_string(), &contents])
            .unwrap();
    })
    .await
    .unwrap();

    Ok(contents_copy)
}

pub async fn get_beatmap(beatmap_id: u32) -> Option<String> {
    // I have no idea what's happening here either
    let result = task::spawn_blocking(move || {
        let connection = Connection::open("./data.db").unwrap();
        let query = "SELECT * FROM maps WHERE id = ?1";
        let mut stmt = connection.prepare(query).unwrap();
        let mut rows = stmt.query([&beatmap_id]).unwrap();

        if let Some(first_row) = rows.next().unwrap_or(None) {
            return first_row.get("data").ok();
        }

        None
    })
    .await
    .unwrap();

    if let Some(data) = result {
        return Some(data);
    }

    // If the data is not found in the database, download it
    let downloaded_contents = download_beatmap_and_give_data(beatmap_id).await;

    match downloaded_contents {
        Ok(contents) => Some(contents),
        Err(_) => None,
    }
}

pub async fn get_performance(
    beatmap_id: u32,
    ruleset: GameMode,
    mods_bits: u32,
    max_combo: u32,
    statistics: &ScoreStatistics,
) -> GetPerformance {
    // Only way this will return an error is if the beatmap doesn't exist.
    // So I just unwrap it because the edge case is super insanely rare
    let map_data = get_beatmap(beatmap_id).await.unwrap();
    let converted_ruleset = match ruleset {
        GameMode::Mania => RosuGameMode::Mania,
        GameMode::Taiko => RosuGameMode::Taiko,
        GameMode::Catch => RosuGameMode::Catch,
        _ => RosuGameMode::Osu,
    };

    let mut map = Beatmap::from_bytes(map_data.as_bytes()).unwrap();

    // Convert beatmap
    map.convert_in_place(converted_ruleset);

    let diff_attrs = Difficulty::new().mods(mods_bits).calculate(&map);

    let max_perf = Performance::new(diff_attrs.clone()).calculate();
    let curr_perf = Performance::new(diff_attrs.clone())
        .n_geki(statistics.perfect)
        .n300(statistics.great)
        .n_katu(statistics.good)
        .n100(statistics.ok)
        .n50(statistics.meh)
        .misses(statistics.miss)
        .combo(max_combo)
        .calculate();
    let fc_perf = Performance::new(diff_attrs.clone())
        .n_geki(statistics.perfect)
        .n300(statistics.great)
        .n_katu(statistics.good)
        .n100(statistics.ok)
        .n50(statistics.meh)
        .misses(0)
        .calculate();

    GetPerformance {
        curr_perf,
        fc_perf,
        max_perf,
        diff_attrs,
        map,
    }
}

// Calculate the accuracy
// Stolen from https://docs.rs/rosu-v2/latest/src/rosu_v2/model/score.rs.html#485-509
// :P
pub fn accuracy(statistics: ScoreStatistics, mode: GameMode) -> f32 {
    let amount_objects = statistics.total_hits(mode) as f32;

    let (numerator, denumerator) = match mode {
        GameMode::Taiko => (
            0.5 * statistics.ok as f32 + statistics.great as f32,
            amount_objects,
        ),
        GameMode::Catch => (
            (statistics.great + statistics.ok + statistics.meh) as f32,
            amount_objects,
        ),
        GameMode::Osu | GameMode::Mania => {
            let mut n = (statistics.meh * 50 + statistics.ok * 100 + statistics.great * 300) as f32;

            n += (u32::from(mode == GameMode::Mania)
                * (statistics.good * 200 + statistics.perfect * 300)) as f32;

            (n, amount_objects * 300.0)
        }
    };

    (10_000.0 * numerator / denumerator).round() / 100.0
}
