mod cli;
mod git;

use cli::{ CliMode, parse_args };

use entry_manager::compare;
use config::*;

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
        create_config(config_path.clone()).await?;
        println!("Done!"); 
        println!("The base config is Non-usable, as it's only a template. \
            Please edit your config file and re-run the program.");
        return Ok(());
    }

    let config = get_config(config_path).await?;
    config.check()?;
    let (name, repository, entries) = config.extract();

    match parse_args() {
        CliMode::Help => cli::help(),
        CliMode::Unknow => { 
            eprintln!("Unknow cli mode");
            cli::help();
        },
        CliMode::Status => {
            println!("Config group is set to [{}]", name);
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

                compare(entry, buf).await.map_err(ConfigError::Io)?;
            }
        }
        _ => todo!()
    };

    Ok(())
}
