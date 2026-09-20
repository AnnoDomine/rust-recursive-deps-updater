#![forbid(unsafe_code)]

pub mod config;

fn main() {
    println!("This application is currently in an early stage and not ready for use!");
    let config = config::model::ModelConfig::new();
    println!("{:?}", config);
}
