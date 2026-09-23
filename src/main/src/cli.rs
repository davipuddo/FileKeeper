use std::env;

use crate::git;

use entry_manager;
use config::{
    ConfigError,
    config_home,
    create_config,
    get_config
};

use std::path::PathBuf;

pub(crate) enum ConfirmDefault {
    Yes,
    No
}

#[derive(Clone, Debug)]
pub(super) enum GitMode {
    Push(String),
    Pull,
    Restore,
    Status,
}

#[derive(Clone, Debug)]
pub(super) enum CliMode {
    Unknow,
    Help,
    Check,
    Status,
    Switch(String),
    Git(GitMode),
    Edit,
    UpdateEntries,  // Machine -> Entries
    UpdateMachine,  // Entries -> Machine
}

pub(super) fn help() {
    let str = String::from(
        ""
    );
    println!("{}", str);
}

fn parse_args() -> CliMode {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);

    if args.is_empty() {
        eprintln!("No arguments provided");
        return CliMode::Help
    }

    match args[0].as_str() {
        "help" | "-h" => CliMode::Help,
        "check" | "changes" | "-c" => CliMode::Check,
        "status" | "-s" => CliMode::Status,
        "update" | "sync" | "-S" => {
            if args.len() < 2 {
                eprintln!("Update requires an additional argument");
                return CliMode::Help
            }
            match args[1].as_str() {
                "machine" | "home" | "." => CliMode::UpdateMachine,
                "entries" => CliMode::UpdateEntries,
                _ => CliMode::Unknow
            }
        }
        "git" | "-g" => {
            if args.len() < 2 {
                eprintln!("No command was provided");
                return CliMode::Help
            }
            match args[1].as_str() {
                "pull" => CliMode::Git(GitMode::Pull),
                "push" => CliMode::Git(GitMode::Push(args[2].clone())),
                "restore" => CliMode::Git(GitMode::Restore),
                "status" => CliMode::Git(GitMode::Status),
                _ => CliMode::Unknow
            }
        },
        "switch" => {
            if args.len() < 2 {
                eprintln!("No group name was provided");
                return CliMode::Help
            }
            CliMode::Switch(args[1].to_string())
        },
        "edit" | "config" | "-e" => {
            CliMode::Edit
        },
        _ => CliMode::Unknow
    }
}

pub async fn execute(config_path: &PathBuf) -> Result<(), ConfigError> {

    let args = parse_args();

    // Execute set of args that do not require a valid config
    let mut stop = true;
    match args.clone() {
        CliMode::Help => help(),
        CliMode::Unknow => { 
            eprintln!("Unknow CLI mode");
            help();
        },
        CliMode::Switch(name) => config::switch_group(name)?,
        _ => stop = false
    }

    if stop {
        return Ok(());
    }

    let config = get_config(config_path.clone())?;
    config.check()?;
    let (repository, entries) = config.extract();

    // Execute other parameters
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
                let entry = PathBuf::from(&entry);
                let buf = PathBuf::from(&repository);
                
                let local = buf.join(&entry.file_name().unwrap());

                let _ = entry_manager::update(&local, &entry);
            }
        }
        CliMode::UpdateEntries => {
            for entry in entries {
                let entry = PathBuf::from(&entry);
                let buf = PathBuf::from(&repository);
                
                let local = buf.join(&entry.file_name().unwrap());

                let _ = entry_manager::update(&entry, &local);
            }
        }
        CliMode::Check => {
            for entry in entries {

                let entry = PathBuf::from(&entry);
                let buf = PathBuf::from(&repository);
                
                let local = buf.join(&entry.file_name().unwrap());

                let _ = entry_manager::compare(&entry, &local);
            }
        },
        CliMode::Edit => {
            open_editor(config_path).await;
        }
        _ => () // Unreachable state
    };
    Ok(())
}

async fn open_editor(config_path: &PathBuf) {

    let config_path = config_path
        .clone()
        .into_string()
        .expect("Could not convert config path to a string");

    let config_file: String = format!("{}/config.toml", config_path)
        .chars()
        .filter(|&x| x != '"')
        .collect();

    let editor = match std::env::var("EDITOR") {
        Ok(editor) => editor,
        Err(_) => {
            eprintln!("EDITOR variable is not set!\nDefaulting to VIM");
            String::from("vim")
        }
    };

    tokio::process::Command::new(&editor)
    .arg(&config_file)
    .status()
    .await
    .expect(&format!("Could not run {} {}", editor, &config_file));
}

pub(crate) fn confirm(prompt: &str, default: ConfirmDefault) -> bool {
    println!("{}", prompt);

    let stop = false;

    while !stop {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .unwrap();

        match buffer.to_lowercase().as_str() {
            "yes" | "y" => return true,
            "no" | "n" => return false,
            "\n" | "" => {
                return match default {
                    ConfirmDefault::Yes => true,
                    ConfirmDefault::No=> false,
                }
            }
            _ => ()
        }
    }
    false
}
