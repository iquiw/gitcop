use std::convert::TryFrom;

use failure::Fail;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;

use indexmap::IndexMap;

use super::types::{GitHub, Repo};

#[derive(Debug, Fail)]
pub enum ConfigError {
    #[fail(display = "invalid repo name: {}", name)]
    InvalidRepo { name: String },
    #[fail(display = "unknown repo type: {}", type_)]
    UnknownType { type_: String },
}

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

impl TryFrom<(&str, &RepoSpec)> for Repo {
    type Error = ConfigError;

    fn try_from((key, val): (&str, &RepoSpec)) -> Result<Self, Self::Error> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^([^/]+)(?:/([^/]+))?$").unwrap();
        }
        let spec = match val {
            RepoSpec::Simple(s) => s,
            RepoSpec::Normal { type_, repo } => {
                if type_ == "github" {
                    repo
                } else {
                    return Err(ConfigError::UnknownType {
                        type_: type_.to_string(),
                    });
                }
            }
        };
        if let Some(cap) = RE.captures(&spec) {
            Ok(Repo::GitHub(GitHub::new(
                cap.get(1).unwrap().as_str(),
                cap.get(2).map(|m| m.as_str()).unwrap_or(key),
            )))
        } else {
            Err(ConfigError::InvalidRepo {
                name: spec.to_string(),
            })
        }
    }
}
