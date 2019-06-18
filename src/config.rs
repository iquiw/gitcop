use std::collections::{hash_map, HashMap};
use std::fmt;
use std::fs::File;
use std::io::Read;
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

impl GitHub {
    fn new<S>(user: S, project: S) -> Self
    where
        S: Into<String>,
    {
        GitHub {
            user: user.into(),
            project: project.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug, Fail, PartialEq)]
pub struct RepoNotFound {
    name: String,
}

impl fmt::Display for RepoNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: Repo not found", Red.paint(&self.name))
    }
}

impl<'a> Iterator for ReposIter<'a> {
    type Item = Result<(&'a str, &'a Repo), RepoNotFound>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ReposIter::Selected(ReposSelected { cfg, names }) => {
                while let Some(n) = names.next() {
                    if let Some(repo) = cfg.repos.get(*n) {
                        return Some(Ok((n, repo)));
                    } else {
                        return Some(Err(RepoNotFound {
                            name: n.to_string(),
                        }));
                    }
                }
                None
            }
            ReposIter::All(ReposAll { iter }) => {
                iter.next().map(|(s, repo)| Ok((s.as_ref(), repo)))
            }
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
                Repo::GitHub(GitHub::new(
                    cap.get(1).unwrap().as_str(),
                    cap.get(2).map(|m| m.as_str()).unwrap_or(key),
                )),
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

    #[test]
    fn test_config_repos_iter_none() {
        let cfg = Config {
            dir: None,
            repos: HashMap::new(),
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_one() {
        let repo = Repo::GitHub(GitHub::new("foo", "bar"));
        let mut repos = HashMap::new();
        repos.insert("one".to_string(), repo.clone());
        let cfg = Config { dir: None, repos };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", &repo))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_multiple() {
        let repo1 = Repo::GitHub(GitHub::new("foo1", "bar1"));
        let repo2 = Repo::GitHub(GitHub::new("foo2", "bar2"));
        let repo3 = Repo::GitHub(GitHub::new("foo3", "bar3"));
        let mut repos = HashMap::new();
        repos.insert("one".to_string(), repo1.clone());
        repos.insert("two".to_string(), repo2.clone());
        repos.insert("three".to_string(), repo3.clone());
        let cfg = Config { dir: None, repos };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", &repo1))));
        assert_eq!(iter.next(), Some(Ok(("two", &repo2))));
        assert_eq!(iter.next(), Some(Ok(("three", &repo3))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_none_selected() {
        let cfg = Config {
            dir: None,
            repos: HashMap::new(),
        };
        let names = vec!["one"];
        let mut iter = cfg.repos(Some(&names));
        assert_eq!(
            iter.next(),
            Some(Err(RepoNotFound {
                name: "one".to_string()
            }))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_multiple_selected() {
        let repo1 = Repo::GitHub(GitHub::new("foo1", "bar1"));
        let repo2 = Repo::GitHub(GitHub::new("foo2", "bar2"));
        let repo3 = Repo::GitHub(GitHub::new("foo3", "bar3"));
        let mut repos = HashMap::new();
        repos.insert("one".to_string(), repo1.clone());
        repos.insert("two".to_string(), repo2.clone());
        repos.insert("three".to_string(), repo3.clone());
        let cfg = Config { dir: None, repos };

        let names = vec!["one", "three"];
        let mut iter = cfg.repos(Some(&names));
        assert_eq!(iter.next(), Some(Ok(("one", &repo1))));
        assert_eq!(iter.next(), Some(Ok(("three", &repo3))));
        assert_eq!(iter.next(), None);
    }
}
