use serde::Deserialize;

use std::path::PathBuf;

use tokio::{
    fs::{self},
    io::{self}
};

use directories::ProjectDirs;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    name: String,
    repository: String,
    directories: Vec<String>
}

impl Config {
    pub fn extract(self) -> (String, String, Vec<String>) {
        (self.name, self.repository, self.directories)
    }
}

#[derive(Deserialize, Debug)]
struct Groups {
    group: Vec<Config>
}

#[derive(Debug)]
pub enum ConfigError {
    NoConfigHome,
    ParseError,
    InvalidId(usize),
    Io(io::Error)
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
directories = ["{}"]"#,
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
            if let Some (val) = config.group.get(id) {
                Ok(val.clone())
            } else {
                Err(ConfigError::InvalidId(id))
            }
        },
        Err(_) => Err(ConfigError::ParseError)
    }
}
