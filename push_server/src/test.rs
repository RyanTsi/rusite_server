use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Deserialize, Debug)]
struct point {
    x: i32,
    content: String,
}
#[tokio::test]
async fn solve() -> Result<()>{
    // initialize();
    // let pool = build_pool().await.unwrap();
    // let x = dbops::query_essays_last_save_time(&pool).await?;
    // println!("{:?}", x);
    // for i in 0..10 {
    //     let uuid = Uuid::new_v4().to_string();
    //     println!("{}", uuid);
    // }
    Ok(())
}