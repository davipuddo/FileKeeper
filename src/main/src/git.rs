use crate::cli::{ ConfirmDefault, confirm };

use std::{
    process::Command,
    io::Result,
    path::PathBuf
};

#[derive(Clone, Debug)]
pub(super) enum GitMode {
    Push(String),
    Pull,
    Restore,
    Status,
}

struct Git { 
    path: String 
}

impl Git {
    fn command(&self, args: Vec<&str>) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .output()?;

        let out = match String::from_utf8(output.stdout) {
            Ok(str) => str,
            Err(e) => { panic!("{}", e); }
        };

        let err = match String::from_utf8(output.stderr) {
            Ok(str) => str,
            Err(e) => { panic!("{}", e); }
        };

        let res = format!("{}\n{}", out, err);

        Ok(res)
    }
}

pub(super) fn is_single_repository(groups: &Vec<(String, Vec<String>)>) -> Result<Option<PathBuf>> {

    let (keep_1, _) = &groups[0];
    let (keep_2, _) = &groups[1];

    let mut path_1 = PathBuf::from(keep_1);
    let mut path_2 = PathBuf::from(keep_2);

    if path_1 == path_2 {
        return Ok(Some(path_1));
    }

    let mut stop = false;
    let mut single_repo = false;

    while !stop {
        path_1.push(".git");
        path_2.push(".git");

        match (path_1.try_exists()?, path_2.try_exists()?) {
            (false, true) | (true, false) => { stop = true; },
            (true, true) => {
                if path_1 == path_2 {
                    stop = true;
                    single_repo = true;
                }
            }
            (false, false) => ()
        }

        // Remove git
        path_1.pop();
        path_2.pop();

        if !stop {
            // Go to parents 
            path_1.pop();
            path_2.pop();
        }
    }

    if single_repo {
        Ok(Some(path_1))
    } else {
        Ok(None)
    }
}

pub(super) fn handle (mode: &GitMode, path: String) -> Result<()> {

    let git = Git { path };

    match mode {
        GitMode::Status => {
            let out = git.command(vec!["status"]).unwrap();
            println!("{}",out);
        },
        GitMode::Push(msg) => {
            let out = git.command(vec!["add", "."])?;
            println!("{}", out);

            let out = git.command(vec!["commit", "-m", &msg])?;
            println!("{}", out);

            let out = git.command(vec!["push"])?;
            println!("{}", out);
        },
        GitMode::Pull => {
            let out = git.command(vec!["pull"])?;
            println!("{}", out);
        },
        GitMode::Restore => {
            let prompt = format!("Are you sure you want to restore the \
                changes made to the repository at [{}]", &git.path);
            
            match confirm(&prompt, ConfirmDefault::No) {
                false => println!("Aborting restore!"),
                true => {
                    let out = git.command(vec!["restore", "."])?;
                    println!("{}", out);
                }
            }
        }
    };

    Ok(())
}
