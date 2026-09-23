#![allow(unused)]

mod cli;

mod git;
use entry_manager;
use config::{
    ConfigError,
    config_home,
    create_config,
    get_config
};

use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), ConfigError> {

    let config_path = match config_home() {
        Ok(config) => config,
        Err(e) => panic!("{:?}", e)
    };

    // Create a new config if there is none
    if !config_path.try_exists().map_err(ConfigError::Io)? {
        println!("No config detected. Creating base config at: [{:?}]", &config_path); 
        create_config(config_path.clone())?;
        println!("Done!\n"); 
        println!("The base config is Non-usable, as it's only a template. \
            Please edit your config file and re-run the program.");

        return Ok(())
    }

    cli::execute(&config_path).await?;

    Ok(())
}
