use std::time::Duration;

use anyhow::Result;
use sqlx::{
    MySql,
    Pool,
    mysql::MySqlPoolOptions,
};

use crate::DATABASE_URL;

pub async fn build_pool() -> Result<Pool<MySql>> {
    Ok(MySqlPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&DATABASE_URL)
        .await
        .expect("can't connect database")
    )
}