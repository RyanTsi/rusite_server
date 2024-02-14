use std::fs;
use std::io::{BufRead, BufReader};

/// 得到 `dir` 文件夹下所有后缀为 `suffix` 的文件路径。
pub fn get_entries(dir: &str, suffix: &str) -> Vec<String> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut res:Vec<String> = Vec::new();
            for i in entries {
                let path = i.unwrap().path();
                if path.is_dir() {
                    res.append(&mut get_entries(path.to_str().unwrap(), suffix));
                } else if path.extension().unwrap() == suffix {
                    res.push(path.to_str().unwrap().to_string());
                }
            }
            res
        },
        Err(_) => Vec::new(),
    }
}

use lazy_static::lazy_static;
use pulldown_cmark::html;
use pulldown_cmark::{Options, Parser};

use crate::data_struct::Essay;

/// 通过文件的路径转化为 Essay 内容。
pub fn get_essay(path: &str) -> Option<Essay> {
    match fs::File::open(path) {
        Ok(file) => {
            let mut yaml = Vec::new();
            let mut md_content = Vec::new();
            let mut in_yaml_block = false;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                if line.trim().eq("---") {
                    in_yaml_block = ! in_yaml_block;
                } else {
                    if in_yaml_block {
                        yaml.push(line);
                    } else {
                        md_content.push(line);
                    }
                }
            }
            let yaml = yaml.join("\n");
            let md_content = MDRENDERER.render(&md_content.join("\n"));
            Some(Essay::from(&yaml, &md_content))
        }
        Err(_) => None,
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
    pub fn render(&self, md_content: &str) -> String {
        let parser = Parser::new_ext(md_content, self.option);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    }
}

lazy_static! {
    pub static ref MDRENDERER: MarkdownRenderer = {
        MarkdownRenderer::new()
    };
}