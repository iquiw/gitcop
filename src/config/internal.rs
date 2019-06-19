use serde::Deserialize;

use indexmap::IndexMap;

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
    pub directory: Option<String>,
    pub repositories: IndexMap<String, RepoSpec>,
}
