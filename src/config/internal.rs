use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RepoSpec {
    Simple(String),
    Normal {
        #[serde(rename = "type")]
        type_: String,
        repo: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct ConfigInternal {
    pub repositories: HashMap<String, RepoSpec>,
}
