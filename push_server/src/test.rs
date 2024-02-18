use std::fs;

use serde::{Deserialize, Serialize};
use crate::Essay;

#[derive(Deserialize, Debug)]
struct point {
    x: i32,
    content: String,
}
#[test]
fn solve() {
    // let yaml = "x: 32".to_string();
    // let content = "12123\n123\n123".to_string();
    // let yaml = yaml + &"\ncontent: \"" + &content + "\"";
    // println!("{}", &yaml);
    // let x: point= serde_yaml::from_str(&yaml).unwrap();
    // println!("{:?}", x);
    let ee = Essay::new("121".to_owned(), "hoeelo".to_owned(), "date".to_owned(), vec!["categories1".to_string(), "c2".to_string()], vec!["tags".to_string(), "t.to_string()2".to_string()], "brief".to_owned(), "12e12\ndasd\n".to_owned());
    println!("{}", serde_yaml::to_string(&ee).unwrap());
}