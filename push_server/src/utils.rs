use std::fs;
use std::io::{BufRead, BufReader};
use std::time::UNIX_EPOCH;

use anyhow::Result;

/// 递归得到 `dir` 文件夹下所有后缀为 `suffix` 的文件路径。
pub fn get_entries(dir: &str, suffix: &str) -> Vec<String> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut res: Vec<String> = Vec::new();
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

pub fn check_modified(path: &str, last_run_time: f64) -> Result<bool> {
    let metadata = fs::metadata(&path)?;
    let modified_time = metadata.modified()?.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    Ok(modified_time > last_run_time)
}