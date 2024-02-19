use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use dbops::{insert_essay, query_essays_last_save_time, update_essay};
use lazy_static::lazy_static;
use tokio::{fs, io::{AsyncBufReadExt, BufReader}};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use uuid::Uuid;
use anyhow::Result;
use dotenv::dotenv;
use std::env;
use utils::*;

mod dbops;
mod utils;
#[cfg(test)]
mod test;

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

lazy_static! {
    static ref DATABASE_URL: String = env::var("DATABASE_URL").expect("DATABASE_URL is not defined");
    static ref CURRENT_TIME: f64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
}

async fn initialize() {
    dotenv().ok();
}

pub async fn build_pool() -> Result<Pool<MySql>> {
    Ok(MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&DATABASE_URL)
        .await?)
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
    
    for essay in &essays {
        if db_essay_last_save_time.contains_key(&essay.eid) {
            if file_essay_last_save_time.get(&essay.eid).unwrap() > db_essay_last_save_time.get(&essay.eid).unwrap() {
                update_essay(&pool, essay, *CURRENT_TIME).await?;
                println!("->> {:<12} {} is updated", "INFO", essay.title);
            }
        } else {
            insert_essay(&pool, essay, *CURRENT_TIME).await?;
            println!("->> {:<12} {} is inserted", "INFO", essay.title);
        }
    }


    // let uuid = Uuid::new_v4().to_string();
    // println!("{}", uuid);

    Ok(())
}


#[derive(Debug, Clone, Deserialize)]
struct EssayInfo {
    eid: String,
    title: String,
    date: String,
    categories: Vec<String>,
    tags: Vec<String>,
    brief: String,
}

/// Essay class
#[derive(Debug, Clone, Deserialize)]
struct Essay {
    eid: String,
    title: String,
    date: String,
    categories: Vec<String>,
    tags: Vec<String>,
    brief: String,
    content: String,
}

impl Essay {
    fn new(
        eid: String,
        title: String,
        date: String,
        categories: Vec<String>,
        tags: Vec<String>,
        brief: String,
        content: String,
    ) -> Self {
        Self {
            eid, title, date, categories, tags, brief, content,
        }
    }
    /// 从 markdown 文件路径得到一个 Essay class
    async fn crate_from_path(
        path: &str,
    ) -> Result<Self> {
        let file = fs::File::open(path).await?;
        let mut yaml = Vec::new();
        let mut md_content = Vec::new();
        let mut in_yaml_block = false;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await.transpose() {
            let line = line?;
            if line.trim().eq("---") {
                in_yaml_block = !in_yaml_block;
            } else {
                if in_yaml_block {
                    yaml.push(line)
                } else {
                    md_content.push(line)
                }
            }
        }
        let content = MarkdownRenderer::new().render(&md_content.join("\n")).await;
        let yaml = yaml.join("\n");
        let essay_info: EssayInfo = serde_yaml::from_str(&yaml).expect(&(String::from(path) + " yaml don't accepted\n" + &yaml));
        let mut res = Self::from(essay_info);
        res.content = content;
        Ok(res)
    }
}

impl From<EssayInfo> for Essay {
    fn from(essay_info: EssayInfo) -> Self {
        Self {
            eid: essay_info.eid,
            title: essay_info.title,
            date: essay_info.date,
            categories: essay_info.categories,
            tags: essay_info.tags,
            brief: essay_info.brief,
            content: Default::default(),
        }
    }
}

/// markdown 渲染器
pub struct MarkdownRenderer {
    option: Options,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let mut res = Self {
            option: Options::empty()
        };
        res.option.insert(Options::ENABLE_STRIKETHROUGH);
        res
    }
    pub async fn render(&self, md_content: &str) -> String {
        let parser = Parser::new_ext(md_content, self.option);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}
