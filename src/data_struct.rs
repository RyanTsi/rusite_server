use std::collections::BTreeMap;
use crate::utils::MDRENDERER;
use serde::{Deserialize, Serialize};
use chrono::DateTime;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EssayAmb {
    pub title: String,
    pub date: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub excerpt: String,
    priority: u32
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct EssayContent {
    content: String,
}

impl EssayContent {
    pub fn from(content:&str) -> Self {
        Self {
            content: MDRENDERER.render(&content)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Essay {
    pub amb: EssayAmb,
    pub content: EssayContent
}
impl Essay {
    pub fn from(amb: &str ,content: &str) -> Self {
        Self {
            amb: serde_yaml::from_str(amb).unwrap(),
            content: EssayContent::from(content)
        }
    }
    pub fn get_unix(&self) -> i64 {
        println!("{}",&self.amb.date);
        let mut ss = (&self).amb.date.clone();
        ss += " +0800";
        // 将字符串解析为 DateTime 对象
        let parsed_date = DateTime::parse_from_str(&ss, "%Y-%m-%d %H:%M:%S %z")
            .expect("Failed to parse date string");
        // 将 DateTime 对象转换为时间戳（Unix 时间戳）
        let timestamp = parsed_date.timestamp();
        timestamp
    }
}

pub struct Essays {
    essays: BTreeMap<i64, Essay>
}
impl Essays {
    pub fn new() -> Self {
        Self { essays: BTreeMap::new() }
    }
    pub fn add_essay(&mut self, essay: Essay) {
        // let ttt = essay.get_unix();
        self.essays.insert(essay.get_unix(), essay);
    }
    pub fn del_essay(&mut self, key: i64) {
        self.essays.remove(&key);
    }
    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, i64, Essay>{
        self.essays.iter()
    }
}