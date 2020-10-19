use std::convert::TryFrom;
use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::Regex;
use serde::de;
use serde::{Deserialize, Deserializer};

use indexmap::IndexMap;

use super::types::{GitCmd, GitHub, Repo};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid repo name: {name:}")]
    InvalidRepo { name: String },
    #[error("unknown repo type: {type_:}")]
    UnknownType { type_: String },
}

#[derive(Debug)]
pub struct Concurrency(u16);

impl Concurrency {
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl Default for Concurrency {
    fn default() -> Self {
        Concurrency(10)
    }
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
    #[serde(default = "GitCmd::default")]
    pub git: GitCmd,
    pub directory: Option<String>,
    #[serde(default)]
    pub concurrency: Concurrency,
    pub repositories: IndexMap<String, RepoSpec>,
    #[serde(rename = "optional-repositories")]
    pub optional_repositories: Option<IndexMap<String, RepoSpec>>,
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

impl<'de> Deserialize<'de> for GitCmd {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = PathBuf::deserialize(d)?;
        Ok(GitCmd::new(&value))
    }
}

impl<'de> Deserialize<'de> for Concurrency {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: u16 = Deserialize::deserialize(d)?;
        if value == 0 {
            return Err(de::Error::invalid_value(
                de::Unexpected::Unsigned(0),
                &"positive integer",
            ));
        }
        Ok(Concurrency(value))
    }
}
