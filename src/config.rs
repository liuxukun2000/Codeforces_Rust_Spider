use std::io::Read;
use serde_derive::{Deserialize, Serialize};
use toml;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub username: Vec<String>,
    pub password: Vec<String>,
    pub agents: Vec<String>
}


impl Config {
    pub fn load() -> Self {
        let mut file = std::fs::File::open("config.toml").unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).ok();
        toml::from_str(&content).unwrap()
    }

    // pub fn get() ->
}