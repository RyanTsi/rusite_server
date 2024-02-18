use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{fs, io::{AsyncBufReadExt, BufReader}};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySql, MySqlPool, Row};
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
#[derive(Clone,Deserialize, Serialize)]
struct Config {
    last_run: f64,
}

impl Config {
    async fn new() -> Self {
        let config_path = "./config.json";
        let res = match fs::read_to_string("./config.json").await {
            Ok(config_conten) => {
                serde_json::from_str(&config_conten).unwrap()
            },
            Err(_) => {
                Self {
                    last_run: 0.0,
                }
            }
        };
        // now time
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
        println!("{}", current_time);
        let mut new_config = res.clone();
        new_config.last_run = current_time;
        // update last_run
        let _ = fs::write(config_path, serde_json::to_string_pretty(&new_config).unwrap()).await;
        res
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    
    dotenv().ok();

    let config = Config::new().await;

    let pool = MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    // // 执行查询
    // let rows = sqlx::query("SELECT eid, title, brief FROM essays")
    //     .fetch_all(&pool)
    //     .await?;

    // // 打印查询结果
    // for row in rows {
    //     let title: String = row.get("eid");
    //     let brief: String = row.get("brief");
    //     println!("Title: {}, Brief: {}", title, brief);
    // }
    // let uuid = Uuid::new_v4().to_string();
    // println!("{}", uuid);
    
    let essays_path = utils::get_entries("../res", "md");
    let mut essays = Vec::new();
    for essay_path in essays_path {
        essays.push(Essay::crate_from_path(&essay_path).await?);
    }

    for ee in essays {
        let x = dbops::insert_eaasy(&pool, ee).await?;
        println!("{x}");
    }

    Ok(())
}


/// Essay class
#[derive(Debug, Clone, Deserialize, Serialize)]
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
            let line = line.unwrap();
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
        let md_renderer = MarkdownRenderer::new();
        let content = md_renderer
            .render(&md_content.join("\n"))
            .await
            .replace("\n", "\n  ");
        let yaml = yaml.join("\n") + "\ncontent: |\n  " + &content;
        println!("{}", yaml);
        Ok(serde_yaml::from_str(&yaml).unwrap())
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
