use serde::Deserialize;

use std::{
    path::PathBuf,
    fs::{write, File, create_dir_all, read_to_string},
    io,
    collections::HashMap
};

use directories::ProjectDirs;

#[derive(Debug)]
pub enum ConfigError {
    NoConfigHome,

    NoBaseRepositoryProvided,
    InvalidBaseRepository(String),

    NoEntriesProvided,
    InvalidEntry(String),

    ParseError,
    InvalidGroupName(String),
    Io(io::Error)
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    repository: String,
    entries: Vec<String>
}

#[derive(Deserialize, Debug)]
struct Groups {
    group: HashMap<String, Config>
}

impl Config {
    pub fn extract(self) -> (String, Vec<String>) {
        (self.repository, self.entries)
    }

    pub fn check(&self) -> Result<(), ConfigError> {

        if self.repository.is_empty() {
            return Err(ConfigError::NoBaseRepositoryProvided)
        }

        let path_check = PathBuf::from(&self.repository).try_exists();

        if path_check.is_err() || path_check.is_ok_and(|x| x == false) {
            return Err(ConfigError::InvalidBaseRepository(self.repository.clone()))
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

pub fn create_config(mut config_path: PathBuf) -> Result<(), ConfigError> {
    create_dir_all(&config_path)
        .map_err(ConfigError::Io)?;

    config_path.push("config.toml");
    File::create(&config_path)
        .map_err(ConfigError::Io)?;

    let opt = config_path.to_str();
    if let None = opt {
        return Err(ConfigError::ParseError);
    }

    let default = format!(
r#"[group."general"]
repository = "path/to/repository"
entries = ["{}"]"#,
opt.unwrap()
        );

    write(&config_path, default)
        .map_err(ConfigError::Io)?;

    config_path.pop();
    config_path.push("group_name.txt");

    File::create(&config_path)
        .map_err(ConfigError::Io)?;

    Ok(())
}

pub fn get_config(mut config_path: PathBuf) -> Result<Vec<Config>, ConfigError> {

    config_path.push("config.toml");

    let config = read_to_string(&config_path)
        .map_err(ConfigError::Io)?;

    let res: Result<Groups, _> = toml::from_str(&config);

    if res.is_err() {
        return Err(ConfigError::ParseError)
    }
    let groups = res.unwrap();

    config_path.pop();
    config_path.push("group_name.txt");

    let group_name = read_to_string(&config_path)
        .map_err(ConfigError::Io)?
        .chars()
        .take_while(|&c| c != '\n')
        .collect();

    match (groups.group.get(&group_name), groups.group.get("general")) {
        (Some(group), Some(general)) => {
            Ok(vec![group.clone(), general.clone()])
        },
        (Some(val), None) | (None, Some(val)) => Ok(vec![val.clone()]),
        (None, None) => Err(ConfigError::InvalidGroupName(group_name)),
    }
}

pub fn switch_group(name: String) -> Result<(), ConfigError> {

    let mut path = config_home()?;

    path.push("group_name.txt");

    if !path.try_exists().map_err(ConfigError::Io)? 
    {
        panic!("File [{:?}] does not exist!", path);
    }

    let old_name = read_to_string(&path)
        .map_err(ConfigError::Io)?;

    if name == old_name {
        println!("Group is already set to [{}]", name)

    } else {
        write(&path, format!("{name}"))
            .map_err(ConfigError::Io)?;

        println!("Switched to group [{}]", name);
    }
    Ok(())
}
