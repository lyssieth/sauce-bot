use std::{fs::OpenOptions, io::Read, io::Write, path::PathBuf};

use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Config {
    credentials: Credentials,
    settings: Settings,
}

impl Config {
    fn get_path() -> PathBuf {
        if option_env!("CONTAINER") == Some("true") {
            PathBuf::from("/config/config.toml")
        } else {
            PathBuf::from("./config.toml")
        }
    }

    pub const fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn load() -> Self {
        let path = Self::get_path();

        if path.exists() {
            let mut file = OpenOptions::new()
                .read(true)
                .open(path)
                .expect("Unable to open `config.toml` for reading. Please check permissions");

            let mut content = String::new();
            file.read_to_string(&mut content)
                .expect("Unable to read `config.toml`");

            toml::from_str(&content).expect("Unable to parse `config.toml`")
        } else {
            let cfg = Self::default();
            cfg.save();

            cfg
        }
    }

    pub fn save(&self) {
        let path = Self::get_path();
        let content = toml::to_string_pretty(self).expect("Unable to parse Config object");

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .expect("Unable to open `config.toml` for writing");

        file.write_all(content.as_bytes())
            .expect("Unable to write to `config.toml`");
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash, SmartDefault, Serialize, Deserialize)]
pub struct Credentials {
    #[default = "INVALID"]
    token: String,
    #[default = "INVALID"]
    saucenao_api_key: String,
    #[default = "INVALID"]
    fuzzysearch_api_key: String,
}

impl Credentials {
    pub const fn token(&self) -> &String {
        &self.token
    }

    pub const fn saucenao_api_key(&self) -> &String {
        &self.saucenao_api_key
    }

    pub const fn fuzzysearch_api_key(&self) -> &String {
        &self.fuzzysearch_api_key
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq, Hash, SmartDefault, Serialize, Deserialize)]
pub struct Settings {
    #[default = 5]
    top_links: u8,
}

impl Settings {
    pub const fn top_links(&self) -> u8 {
        self.top_links
    }
}
