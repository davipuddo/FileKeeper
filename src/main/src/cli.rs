use std::env;

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
    UpdateEntries,  // Machine -> Entries
    UpdateMachine,  // Entries -> Machine
}

pub(super) fn help() {
    let str = String::from(
        ""
    );
    println!("{}", str);
}

pub(super) fn parse_args() -> CliMode {
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
        _ => CliMode::Unknow
    }
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
