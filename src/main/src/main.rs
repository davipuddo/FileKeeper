use file_system::*;
use config::*;

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
        return Ok(());
    }

    let config = get_config(config_path).await?;
    let (name, repository, directories) = config.extract();

    println!("Config group is set to {}", name);
    println!("Base repository is {}", repository);
    println!("The directories to check are: {:?}", directories);

    Ok(())
}
