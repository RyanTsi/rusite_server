use std::collections::HashMap;
use push_server::{
    data_struct::Essay, dbops::{tables_ops::{delete_essay, insert_essay, query_essays_last_save_time, update_essay}, utils::build_pool},
    CURRENT_TIME,
};
use tokio::fs;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use dotenv::dotenv;
mod utils;
use utils::get_modified_time;

/// 读取 config.json 文件，并修改当前时间到 config::last_run
#[derive(Clone, Deserialize, Serialize)]
struct Config {
    essays_source: String,
}

impl Config {
    async fn new() -> Self {
        let config_path = "./config.json";
        match fs::read_to_string(config_path).await {
            Ok(config_conten) => {
                serde_json::from_str(&config_conten).unwrap()
            },
            Err(_) => {
                let res = Self {
                    essays_source: String::from("./res/_essays")
                };
                fs::write(config_path, serde_json::to_string_pretty(&res).unwrap()).await.unwrap();
                res
            }
        }
    }
}



async fn initialize() {
    dotenv().ok();
}



#[tokio::main]
async fn main() -> Result<()> {
    
    initialize().await;

    let config = Config::new().await;
    let essays_source = config.essays_source;
    let pool = build_pool().await?;

    let db_essay_last_save_time = query_essays_last_save_time(&pool).await?;
    let mut file_essay_last_save_time = HashMap::new();
    let essays_path = utils::get_entries(&essays_source, "md");
    let mut essays = Vec::new();
    for essay_path in essays_path {
        essays.push(Essay::crate_from_path(&essay_path).await.expect(&(essay_path.clone() + " yaml is error")));
        file_essay_last_save_time.insert(essays.last().unwrap().eid.clone(), get_modified_time(&essay_path)?);
    }
    
    for (eid, _) in db_essay_last_save_time.iter() {
        if !file_essay_last_save_time.contains_key(eid) {
            delete_essay(&pool, eid).await?;
            println!("->> {:<12} - {}", "DELETE", eid);
        }
    }

    for essay in &essays {
        if db_essay_last_save_time.contains_key(&essay.eid) {
            if file_essay_last_save_time.get(&essay.eid).unwrap() > db_essay_last_save_time.get(&essay.eid).unwrap() {
                update_essay(&pool, essay, *CURRENT_TIME).await?;
                println!("->> {:<12} - {}", "UPDATE", essay.title);
            }
        } else {
            insert_essay(&pool, essay, *CURRENT_TIME).await?;
            println!("->> {:<12} - {}", "INSERT", essay.title);
        }
    }

    // let uuid = Uuid::new_v4().to_string();
    // println!("{}", uuid);

    Ok(())
}


