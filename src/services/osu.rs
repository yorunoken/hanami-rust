use std::env;

use rosu_v2::Osu;
use serenity::prelude::TypeMapKey;

pub struct OsuClient(pub Osu);

impl TypeMapKey for OsuClient {
    type Value = OsuClient;
}

pub(crate) async fn get_client() -> OsuClient {
    // Get and parse osu! client information
    let osu_client_id: u64 = env::var("OSU_CLIENT_ID")
        .expect("Expected OSU_CLIENT_ID to be defined in environment.")
        .parse()
        .expect("OSU_CLIENT_ID is not a number!");

    let osu_client_secret = env::var("OSU_CLIENT_SECRET")
        .expect("Expected OSU_CLIENT_SECRET to be defined in environment.");

    // Build the osu! client
    let osu_client = Osu::new(osu_client_id, osu_client_secret).await.unwrap();

    OsuClient(osu_client)
}
