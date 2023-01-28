#[macro_use]
extern crate lazy_static;

use crate::config::Config;
use crate::problem::Problem;
use std::env::Args;

mod config;
mod user;
mod problem;

#[tokio::main]
async fn main()  -> Result<(),reqwest::Error> {
    let args = std::env::args().collect::<Vec<String>>();

    // let config = Config::load();
    // println!("{:?}", &config.username);
    // Problem::load(&"prob.toml".to_string()).unwrap();
    // Problem::get_data(1353, &"B".to_string()).await;
    Problem::get_problem(args[1].parse::<i32>().unwrap(), &args[2])
        .await.unwrap().to_jcoder_formant(&format!("{}_{}", &args[1], &args[2]));
    Ok(())
}

// fn main() {
//     println!("Hello, world!");
//     let config = Config::load();
//     println!("{:?}", &config.username);
//     Problem::get_problem(1353, &"B".to_string());
// }
