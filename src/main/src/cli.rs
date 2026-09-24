use std::env;

use crate::git::{self, GitMode};

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

fn help() {

    let options = [
        ("help, -h", "show this menu"),
        ("status, -s", "display current selected repositories and entries"),
        ("check, changes, -c", "check for modifications between the local and keep entries"),
        ("update, sync, -S <WHICH>", "update entries"),
        ("", "  - possible values for updating LOCAL entries are: [\"machine\", \"home\", \".\"]"),
        ("", "  - possible values for updating KEEP entries are: [\"repo\", \"keep\"]"),
        ("switch <NAME>", "switch current group to <NAME>"),
        ("edit", "open the configuration file with the editor defined by $EDITOR"),
        ("git, -g <COMMAND>", "run a git command at the selected repositories"),
        ("", "  - possible commands are: [\"status\", \"pull\", \"push\", \"restore\"]")
    ];

    println!("\n{:<10} fkp [OPTIONS]", "Usage: ");

    println!("\nOptions:");
    for (left, right) in options {
        println!("  {:<30} {}", left, right);
    }
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
                eprintln!("No git command was provided");
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
        CliMode::Edit => {
            open_editor(config_path).await;
        }
        _ => stop = false
    }

    if stop {
        return Ok(());
    }

    let config = get_config(config_path.clone())?;

    let mut groups = vec![];

    for config in config {
        config.check()?;
        groups.push(config.extract());
    }

    // Execute other parameters
    match args {
        CliMode::Status => {
            println!("Base repositories are: ");
            for group in &groups {
                println!("- [{:?}]", group.0);
            }
            println!("\nThe entries to check are: ");
            for group in &groups {
                for entry in &group.1 {
                    println!("- [{:?}]", entry);
                }
            }
        },
        CliMode::Git(mode) => { 
            for group in groups {
                let repository = &group.0;
                git::handle(&mode, repository.clone())
                    .await
                    .map_err(ConfigError::Io)?;
            }
        },
        CliMode::UpdateMachine => {
            let prompt = "Update local files?";
            if confirm(prompt, ConfirmDefault::Yes) {
                for group in groups {
                    let repository = &group.0;
                    let entries = &group.1;
                    for entry in entries {
                        let entry = PathBuf::from(&entry);
                        let buf = PathBuf::from(&repository);
                        
                        let local = buf.join(&entry.file_name().unwrap());

                        let _ = entry_manager::update(&local, &entry);
                    }
                }
            }
        }
        CliMode::UpdateEntries => {
            let prompt = "Update files in the keep?";
            if confirm(prompt, ConfirmDefault::Yes) {
                for group in groups {
                    let repository = &group.0;
                    let entries = &group.1;
                    for entry in entries {
                        let entry = PathBuf::from(&entry);
                        let buf = PathBuf::from(&repository);
                        
                        let local = buf.join(&entry.file_name().unwrap());

                        let _ = entry_manager::update(&entry, &local);
                    }
                }
            }
        }
        CliMode::Check => {
            for group in groups {
                let repository = &group.0;
                let entries = &group.1;
                for entry in entries {
                    let entry = PathBuf::from(&entry);
                    let buf = PathBuf::from(&repository);
                    
                    let local = buf.join(&entry.file_name().unwrap());

                    let _ = entry_manager::compare(&entry, &local);
                }
            }
        },
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

    let confirm_box = match default {
        ConfirmDefault::Yes => "[Y/n]",
        ConfirmDefault::No => "[y/N]"
    };

    println!("{}\n\n{}", prompt, confirm_box);

    let stop = false;

    while !stop {
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .unwrap();

        buffer = buffer
            .chars()
            .filter(|&c| !c.is_whitespace())
            .collect();

        match buffer.to_lowercase().as_str() {
            "yes" | "y" => return true,
            "no" | "n" => return false,
            "\n" | "" => {
                return match default {
                    ConfirmDefault::Yes => true,
                    ConfirmDefault::No=> false,
                }
            }
            _ => println!("aaa")
        }
    }
    false
}
