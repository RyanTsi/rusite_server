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
        .connect(&DATABASE_URL)
        .await?)
}