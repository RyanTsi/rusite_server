use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use tokio::{fs, io::{AsyncBufReadExt, BufReader}};
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct EssayInfo {
    pub eid: String,
    pub title: String,
    pub date: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub brief: String,
}
impl EssayInfo {
    pub fn new(
        eid: String,
        title: String,
        date: String,
        categories: Vec<String>,
        tags: Vec<String>,
        brief: String,
    ) -> Self {
        Self {
            eid, title, date, categories, tags, brief
        }
    }
}
/// Essay class
#[derive(Debug, Clone, Deserialize)]
pub struct Essay {
    pub eid: String,
    pub title: String,
    pub date: String,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub brief: String,
    pub content: String,
}

impl Essay {
    pub fn new(
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
    pub async fn crate_from_path(
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
        let essay_info: EssayInfo = serde_yaml::from_str(&yaml).expect(&(String::from(path) + " yaml don't accepted <<-\n"));
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
