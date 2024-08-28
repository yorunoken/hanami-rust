use serenity::prelude::TypeMapKey;
use sqlx::sqlite::SqlitePool;

pub struct Database(pub SqlitePool);

impl TypeMapKey for Database {
    type Value = Database;
}

pub(crate) async fn connect<T: AsRef<str>>(uri: T) -> Database {
    let pool = SqlitePool::connect(uri.as_ref())
        .await
        .expect("Failed to connect to the database.");

    Database(pool)
}
