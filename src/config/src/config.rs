use serde::Deserialize;

use std::path::PathBuf;

use tokio::{
    fs::{self},
    io::{self}
};

use directories::ProjectDirs;

#[derive(Debug)]
pub enum ConfigError {
    NoConfigHome,

    NoBaseRepositoryProvided,
    InvalidBaseRepository,

    NoEntriesProvided,
    InvalidEntry(String),

    ParseError,
    InvalidId(usize),
    Io(io::Error)
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    name: String,
    repository: String,
    entries: Vec<String>
}

#[derive(Deserialize, Debug)]
struct Groups {
    group: Vec<Config>
}

impl Config {
    pub fn extract(self) -> (String, String, Vec<String>) {
        (self.name, self.repository, self.entries)
    }

    pub fn check(&self) -> Result<(), ConfigError> {

        if self.repository.is_empty() {
            return Err(ConfigError::NoBaseRepositoryProvided)
        }

        let path_check = PathBuf::from(&self.repository).try_exists();

        if path_check.is_err() || path_check.is_ok_and(|x| x == false) {
            return Err(ConfigError::InvalidBaseRepository)
        }

        if self.entries.is_empty() {
            return Err(ConfigError::NoEntriesProvided)
        }

        for dir in &self.entries {
            let path_check = PathBuf::from(&dir).try_exists();

            if  path_check.is_err() || path_check.is_ok_and(|x| x == false) {
                return Err(ConfigError::InvalidEntry(dir.clone()))
            }
        }
        Ok(())
    }
}


pub fn config_home() -> Result<PathBuf, ConfigError> {
    let dir = ProjectDirs::from("org", "FileKeeper", "FileKeeper");

    if dir.is_none() {
        return Err(ConfigError::NoConfigHome)
    }
    Ok(dir.unwrap().config_dir().to_path_buf())
}

pub async fn create_config(mut config_path: PathBuf) -> Result<(), ConfigError> {
    fs::create_dir_all(&config_path)
        .await
        .map_err(ConfigError::Io)?;

    config_path.push("config.toml");
    fs::File::create(&config_path)
        .await
        .map_err(ConfigError::Io)?;

    let opt = config_path.to_str();
    if let None = opt {
        return Err(ConfigError::ParseError);
    }

    let default = format!(
r#"[[group]]
name = "machine 1"
repository = "path/to/repository"
entries = ["{}"]"#,
opt.unwrap()
        );

    fs::write(&config_path, default)
        .await
        .map_err(ConfigError::Io)?;

    config_path.pop();
    config_path.push("group_id.txt");
    fs::File::create(&config_path)
        .await
        .map_err(ConfigError::Io)?;

    fs::write(&config_path, "0")
        .await
        .map_err(ConfigError::Io)?;

    Ok(())
}

pub async fn get_config(mut config_path: PathBuf) -> Result<Config, ConfigError> {

    config_path.push("config.toml");

    let file = fs::read_to_string(&config_path)
        .await
        .map_err(ConfigError::Io)?;

    let res: Result<Groups, _> = toml::from_str(&file);

    if res.is_err() {
        return Err(ConfigError::ParseError)
    }
    let config = res.unwrap();

    config_path.pop();
    config_path.push("group_id.txt");

    let file = fs::read_to_string(&config_path)
        .await
        .map_err(ConfigError::Io)?;

    match file.parse::<usize>() {
        Ok(id) => {
            match config.group.get(id) {
                Some(val) => Ok(val.clone()),
                None => Err(ConfigError::InvalidId(id))
            }
        },
        Err(_) => Err(ConfigError::ParseError)
    }
}
