mod cli;
mod git;

use cli::{ CliMode, parse_args };
use entry_manager::{ compare };
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

    if !config_path.try_exists().map_err(ConfigError::Io)? {
        println!("No config detected. Creating base config at: [{:?}]", &config_path); 
        create_config(config_path.clone())?;
        println!("Done!"); 
        println!("The base config is Non-usable, as it's only a template. \
            Please edit your config file and re-run the program.");
        return Ok(());
    }

    let args = parse_args();

    let mut stop = true;
    match args.clone() {
        CliMode::Help => cli::help(),
        CliMode::Unknow => { 
            eprintln!("Unknow CLI mode");
            cli::help();
        },
        CliMode::Switch(name) => config::switch_group(name)?,
        _ => stop = false
    }

    if !stop {
        let config = get_config(config_path)?;
        config.check()?;
        let (repository, entries) = config.extract();

        match args {
            CliMode::Status => {
                println!("Base repository is [{}]", repository);
                println!("The entries to check are: ");
                for entry in entries {
                    println!("- [{}]", entry);
                }
            },
            CliMode::Git(mode) => { 
                git::handle(mode, repository)
                    .await
                    .map_err(ConfigError::Io)?;
            },
            CliMode::UpdateMachine => {
                for entry in entries {
                    
                }
            }
            CliMode::Check => {
                for entry in entries {

                    let entry = PathBuf::from(&entry);
                    let mut buf = PathBuf::from(&repository);
                    
                    buf = buf.join(&entry.file_name().unwrap());

                    let _ = compare(&entry, &buf).await;
                }
            }
            _ => todo!()
        };
    }
    Ok(())
}
