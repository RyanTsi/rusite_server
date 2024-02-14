pub mod utils;
pub mod data_struct;
pub mod router;
pub mod fallback;
pub mod error;


#[cfg(test)]
mod test {
    use std::{fs::File, io::Write};

    use crate::{data_struct::Essays, utils::{get_entries, get_essay}};

    #[test]
    fn get_essays() {
        let path = "./res/_essays";
        let outpath = String::from("public/");
        let mut essays = Essays::new();
        let mds = get_entries(path, "md");
        println!("{:?}", mds);
        for md in mds {
            essays.add_essay(get_essay(&md).unwrap());
        }
        for (id, essay) in essays.iter() {
            let content = serde_json::to_string(essay).unwrap();
            let op = outpath.clone() + id.to_string().as_str() + ".json";
            let mut file = File::create(op).unwrap();
            let _ = file.write_all(content.as_bytes());
        }
    }
}