use std::{collections::HashSet, fs::OpenOptions, io::Read, io::Write, path::PathBuf};

use serde::{Deserialize, Serialize};
use serenity::model::id::UserId;
use smart_default::SmartDefault;

const FILE: &str = "config.toml";

#[derive(Debug, Clone, PartialOrd, PartialEq, Hash, Default, Serialize, Deserialize)]
pub(crate) struct Config {
    credentials: Credentials,
    settings: Settings,
}

impl Config {
    pub(crate) fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    pub(crate) fn settings(&self) -> &Settings {
        &self.settings
    }

    pub(crate) fn load() -> Self {
        let path = PathBuf::from(FILE);
        let conf = if path.exists() {
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
        };

        conf
    }

    pub(crate) fn save(&self) {
        let path = PathBuf::from(FILE);
        let content = toml::to_string_pretty(self).expect("Unable to parse Config object");

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .expect("Unable to open `config.toml` for writing");

        file.write_all(content.as_bytes())
            .expect("Unable to write to `config.toml`");
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Hash, SmartDefault, Serialize, Deserialize)]
pub(crate) struct Credentials {
    #[default = "INVALID"]
    token: String,
    #[default = "INVALID"]
    saucenao_api_key: String,
}

impl Credentials {
    pub(crate) fn token(&self) -> &String {
        &self.token
    }

    pub(crate) fn saucenao_api_key(&self) -> &String {
        &self.saucenao_api_key
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Hash, SmartDefault, Serialize, Deserialize)]
pub(crate) struct Settings {
    #[default = "sauce!"]
    prefix: String,
    #[default = true]
    use_embeds: bool,
    #[default = 5]
    top_links: u8,
    #[default(_code = "vec![]")]
    owner_ids: Vec<u64>,
}

impl Settings {
    pub(crate) fn prefix(&self) -> &String {
        &self.prefix
    }

    pub(crate) fn use_embeds(&self) -> bool {
        self.use_embeds
    }

    pub(crate) fn top_links(&self) -> u8 {
        self.top_links
    }

    pub(crate) fn owner_ids_set(&self) -> HashSet<UserId> {
        self.owner_ids.iter().copied().map(UserId).collect()
    }
}
