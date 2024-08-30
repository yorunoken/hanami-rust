use std::collections::HashMap;

use regex::Regex;
use rosu_v2::prelude::GameModsIntermode;

pub fn get_flags(args: Vec<&str>) -> (HashMap<&str, &str>, Vec<&str>) {
    let mut hash = HashMap::new();
    let mut remaining_args = args.clone();

    let mut i = 0;
    while i < args.len() {
        let parts: Vec<&str> = args[i].split('=').collect();

        if parts.len() == 2 {
            let key = parts[0];
            let value = parts[1];
            hash.insert(key, value);

            remaining_args.remove(i);
        }
        i += 1;
    }

    (hash, remaining_args)
}

pub fn get_mods(args: Vec<&str>) -> (Vec<&str>, Vec<&str>) {
    let mut mods = Vec::new();
    let mut remaining_args = args.clone();

    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("+") {
            let game_mod = &args[i][1..];
            if GameModsIntermode::try_from_acronyms(game_mod).is_some() {
                mods.push(&args[i][1..]);
                remaining_args.remove(i);
            }
        }
        i += 1;
    }

    (mods, remaining_args)
}

pub struct BeatmapURL {
    pub url: String,
    pub id: String,
}

fn parse_url(url: &str) -> Option<BeatmapURL> {
    let regex = Regex::new(r"^https://osu\.ppy\.sh/b/(\d+)").unwrap();

    if let Some(caps) = regex.captures(url) {
        let id = caps.get(1).unwrap().as_str().to_string();

        Some(BeatmapURL {
            url: url.to_string(),
            id,
        })
    } else {
        None
    }
}

pub fn get_beatmap_link(args: Vec<&str>) -> (Option<BeatmapURL>, Vec<&str>) {
    let mut remaining_args = args.clone();

    let mut i = 0;
    let mut beatmap: Option<BeatmapURL> = None;
    while i < args.len() {
        if let Some(b) = parse_url(args[i]) {
            beatmap = Some(b);
            remaining_args.remove(i);
            break;
        }
        i += 1;
    }

    (beatmap, remaining_args)
}

pub fn get_username(args: Vec<&str>) -> Option<String> {
    let mut i = 0;

    while i < args.len() {
        let quote_regex = Regex::new(r#""([^"]+)""#).unwrap();
        if let Some(caps) = quote_regex.captures(args[i]) {
            if let Some(quoted_username) = caps.get(1) {
                return Some(quoted_username.as_str().to_string());
            }
        }
        i += 1;
    }

    if args.len() >= 1 {
        Some(args[0].to_string())
    } else {
        None
    }
}
