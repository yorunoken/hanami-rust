use rosu_v2::model::GameMode;
use rusqlite::Connection;
use serenity::all::UserId;

struct DatabaseUser {
    id: Option<String>,
    bancho_id: Option<u32>,
    mode: Option<String>,
}

#[derive(Debug)]
pub struct User {
    pub bancho_id: u32,
    pub mode: GameMode,
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
