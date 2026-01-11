use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub output: OutputConfig,
    pub input: InputConfig
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct OutputConfig {
    pub redis: Option<RedisConfig>,
    pub console: Option<ConsoleConfig>,
    pub file: Option<FileConfig>,
    pub rmq: Option<RabbitMQConfig>,
    pub postgres: Option<PostgresConfig>
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct InputConfig {
    pub url: String,
    pub workers: usize,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct RedisConfig {
    pub enabled: bool,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ConsoleConfig {
    pub enabled: bool,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct FileConfig {
    pub enabled: bool,
    pub path: Option<String>,
    pub maxfiles: Option<usize>,
    pub threshold: Option<usize>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct RabbitMQConfig {
    pub enabled: bool,
    pub exchange_name: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PostgresConfig {
    pub enabled: bool,
    pub table_name: Option<String>,
    pub system_table_name: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Default for OutputConfig {
    fn default() -> Self {
        OutputConfig { 
            redis: None,
            console: Some(ConsoleConfig::default()),
            file: None,
            rmq: None,
            postgres: None
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        InputConfig { 
            url: "https://www.nationstates.net/api/all".into(),
            workers: 2 
        }
    }
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        ConsoleConfig { 
            enabled: true, 
            include: None,
            exclude: None
        }
    }
}