use std::collections::{hash_map, HashMap};
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::{Path, PathBuf};
use std::slice;

use ansi_term::Colour::Red;
use failure::{Error, Fail};
use lazy_static::lazy_static;
use regex::Regex;

mod internal;
use self::internal::{ConfigInternal, RepoSpec};

#[derive(Debug, Fail)]
enum ConfigError {
    #[fail(display = "invalid repo name: {}", name)]
    InvalidRepo { name: String },
    #[fail(display = "unknown repo type: {}", type_)]
    UnknownType { type_: String },
}

#[derive(Clone, Debug, PartialEq)]
pub struct GitHub {
    pub user: String,
    pub project: String,
}

#[derive(Clone, Debug)]
pub enum Repo {
    GitHub(GitHub),
}

pub trait Remote: std::fmt::Debug {
    fn url(&self) -> String;
}

impl Remote for GitHub {
    fn url(&self) -> String {
        let mut url = String::from("https://github.com/");
        url.push_str(&self.user);
        url.push('/');
        url.push_str(&self.project);
        url.push_str(".git");
        url
    }
}

impl Remote for Repo {
    fn url(&self) -> String {
        match self {
            Repo::GitHub(repo) => repo.url(),
        }
    }
}

#[derive(Debug)]
pub struct Config {
    dir: Option<PathBuf>,
    repos: HashMap<String, Repo>,
}

impl Config {
    pub fn dir(&self) -> Option<&PathBuf> {
        self.dir.as_ref()
    }

    pub fn get(&self, key: &str) -> Option<&Repo> {
        self.repos.get(key)
    }

    pub fn repos<'a>(&'a self, names: Option<&'a Vec<&'a str>>) -> ReposIter<'a> {
        if let Some(names) = names {
            ReposIter::Selected(ReposSelected {
                cfg: self,
                names: names.iter(),
            })
        } else {
            ReposIter::All(ReposAll {
                iter: self.repos.iter(),
            })
        }
    }
}

pub struct ReposAll<'a> {
    iter: hash_map::Iter<'a, String, Repo>,
}

pub struct ReposSelected<'a> {
    cfg: &'a Config,
    names: slice::Iter<'a, &'a str>,
}

pub enum ReposIter<'a> {
    Selected(ReposSelected<'a>),
    All(ReposAll<'a>),
}

impl<'a> Iterator for ReposIter<'a> {
    type Item = (&'a str, &'a Repo);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ReposIter::Selected(ReposSelected { cfg, names }) => {
                while let Some(n) = names.next() {
                    if let Some(repo) = cfg.repos.get(*n) {
                        return Some((n, repo));
                    } else {
                        let stdout = stdout();
                        let mut handle = stdout.lock();
                        writeln!(&mut handle, "{}: Repo not found", Red.paint(*n)).unwrap();
                    }
                }
                None
            }
            ReposIter::All(ReposAll { iter }) => iter.next().map(|(s, repo)| (s.as_ref(), repo)),
        }
    }
}

pub fn load_config<P>(path: P) -> Result<Config, Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    parse_config(&contents)
}

pub fn parse_config(s: &str) -> Result<Config, Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([^/]+)(?:/([^/]+))?$").unwrap();
    }
    let cfgi = toml::from_str::<ConfigInternal>(s)?;
    let dir = cfgi.directory;
    let mut repo_map: HashMap<String, Repo> = HashMap::new();
    for (key, val) in &cfgi.repositories {
        let spec = match val {
            RepoSpec::Simple(s) => s,
            RepoSpec::Normal { type_, repo } => {
                if type_ == "github" {
                    repo
                } else {
                    return Err(Error::from(ConfigError::UnknownType {
                        type_: type_.to_string(),
                    }));
                }
            }
        };
        if let Some(cap) = RE.captures(&spec) {
            repo_map.insert(
                key.to_string(),
                Repo::GitHub(GitHub {
                    user: cap.get(1).unwrap().as_str().to_string(),
                    project: cap
                        .get(2)
                        .map(|m| m.as_str().to_string())
                        .unwrap_or(key.to_string()),
                }),
            );
        } else {
            return Err(Error::from(ConfigError::InvalidRepo {
                name: spec.to_string(),
            }));
        }
    }
    Ok(Config {
        dir: dir.map(|d| PathBuf::from(d)),
        repos: repo_map,
    })
}

#[cfg(test)]
mod test {
    use crate::config::*;
    #[test]
    fn test_parse_config_normal_form() {
        let s = r#"[repositories]
f.type = "github"
f.repo = "rejeep/f.el"

s = { type = "github", repo = "magnars/s.el" }

[repositories.use-package]
type = "github"
repo = "jweigley"

[repositories.dash]
type = "github"
repo = "magnars/dash.el"
"#;
        let cfg = parse_config(s).unwrap();

        assert_eq!(cfg.dir(), None);

        let opt1 = cfg.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");

        let opt2 = cfg.get("dash");
        assert_eq!(opt2.is_some(), true);
        let repo2 = opt2.unwrap();
        assert_eq!(repo2.url(), "https://github.com/magnars/dash.el.git");

        let opt3 = cfg.get("f");
        assert_eq!(opt3.is_some(), true);
        let repo3 = opt3.unwrap();
        assert_eq!(repo3.url(), "https://github.com/rejeep/f.el.git");

        let opt4 = cfg.get("s");
        assert_eq!(opt4.is_some(), true);
        let repo4 = opt4.unwrap();
        assert_eq!(repo4.url(), "https://github.com/magnars/s.el.git");
    }

    #[test]
    fn test_parse_config_simple_form() {
        let s = r#"[repositories]
use-package = "jweigley"
dash = "magnars/dash.el"
"#;
        let cfg = parse_config(s).unwrap();

        assert_eq!(cfg.dir(), None);

        let opt1 = cfg.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");

        let opt2 = cfg.get("dash");
        assert_eq!(opt2.is_some(), true);
        let repo2 = opt2.unwrap();
        assert_eq!(repo2.url(), "https://github.com/magnars/dash.el.git");
    }

    #[test]
    fn test_parse_config_with_directory() {
        let s = r#"directory = "repos"
[repositories]
use-package = "jweigley"
"#;
        let cfg = parse_config(s).unwrap();

        assert_eq!(cfg.dir(), Some(&PathBuf::from("repos")));

        let opt1 = cfg.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");
    }

    #[test]
    fn test_parse_config_invalid_repo() {
        let s = r#"repositories.foo = "bar/baz/foo""#;
        let result = parse_config(s);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            format!("{}", result.err().unwrap()),
            "invalid repo name: bar/baz/foo"
        );
    }

    #[test]
    fn test_parse_config_unknown_type() {
        let s = r#"repositories.foo = { type = "bitbucket", repo = "bar/baz" }"#;
        let result = parse_config(s);

        assert_eq!(result.is_err(), true);
        assert_eq!(
            format!("{}", result.err().unwrap()),
            "unknown repo type: bitbucket"
        );
    }
}
