pub mod data_struct;
pub mod dbops;

use lazy_static::lazy_static;
use std::{env, time::{SystemTime, UNIX_EPOCH}};

lazy_static! {
    pub static ref DATABASE_URL: String = env::var("DATABASE_URL").expect("DATABASE_URL is not defined");
    pub static ref CURRENT_TIME: f64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
}

#[cfg(test)]
mod test;