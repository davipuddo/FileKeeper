use crate::cli::{ ConfirmDefault, confirm };

use tokio::{
    process::Command,
    io
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
    async fn command(&self, args: Vec<&str>) -> Result<String, io::Error> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .output()
            .await?;

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


pub(super) async fn handle (mode: &GitMode, path: String) -> Result<(), io::Error> {

    let git = Git { path };

    match mode {
        GitMode::Status => {
            let out = git.command(vec!["status"]).await?;
            println!("{}", out);
        },
        GitMode::Push(msg) => {
            git.command(vec!["add", "."]).await?;
            git.command(vec!["commit", "-m", &msg]).await?;
            let out = git.command(vec!["push"]).await?;
            println!("{}", out);
        },
        GitMode::Pull => {
            let out = git.command(vec!["pull"]).await?;
            println!("{}", out);
        },
        GitMode::Restore => {
            let prompt = format!("Are you sure you want to restore the \
                changes made to the repository at [{}]", &git.path);
            
            match confirm(&prompt, ConfirmDefault::No) {
                false => println!("Aborting restore!"),
                true => {
                    let out = git.command(vec!["restore", "."]).await?;
                    println!("{}", out);
                }
            }
        }
    };
    Ok(())
}
