use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::slice;

use ansi_term::Colour::Red;
use failure::{Error, Fail};
use indexmap::{self, IndexMap};

mod internal;
mod types;
use self::internal::ConfigInternal;
pub use self::types::{GitHub, Remote, Repo, RepoKind};

#[derive(Debug)]
pub struct Config {
    dir: Option<PathBuf>,
    repos: IndexMap<String, RepoKind>,
}

impl Config {
    pub fn dir(&self) -> Option<&PathBuf> {
        self.dir.as_ref()
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
    iter: indexmap::map::Iter<'a, String, RepoKind>,
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
    type Item = Result<(&'a str, &'a RepoKind), RepoNotFound>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ReposIter::Selected(ReposSelected { cfg, names }) => {
                while let Some(n) = names.next() {
                    if let Some(repo) = cfg.repos.get(*n) {
                        return Some(Ok((n, repo)));
                    } else {
                        return Some(Err(RepoNotFound { name: n.to_string() }));
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
    let cfgi = toml::from_str::<ConfigInternal>(s)?;
    let dir = cfgi.directory;
    let mut repo_map = IndexMap::new();
    for (key, val) in &cfgi.repositories {
        let repo = Repo::try_from((key.as_str(), val))?;
        repo_map.insert(key.to_string(), RepoKind::Default(repo));
    }
    if let Some(opt_repos) = &cfgi.optional_repositories {
        for (key, val) in opt_repos {
            let repo = Repo::try_from((key.as_str(), val))?;
            repo_map.insert(key.to_string(), RepoKind::Optional(repo));
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

        let opt1 = cfg.repos.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");

        let opt2 = cfg.repos.get("dash");
        assert_eq!(opt2.is_some(), true);
        let repo2 = opt2.unwrap();
        assert_eq!(repo2.url(), "https://github.com/magnars/dash.el.git");

        let opt3 = cfg.repos.get("f");
        assert_eq!(opt3.is_some(), true);
        let repo3 = opt3.unwrap();
        assert_eq!(repo3.url(), "https://github.com/rejeep/f.el.git");

        let opt4 = cfg.repos.get("s");
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

        let opt1 = cfg.repos.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");

        let opt2 = cfg.repos.get("dash");
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

        let opt1 = cfg.repos.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");
    }

    #[test]
    fn test_parse_config_with_optional_repos() {
        let s = r#"[repositories]
use-package = "jweigley"
[optional-repositories]
forge = "magit"
magit = "magit"
"#;
        let cfg = parse_config(s).unwrap();

        assert_eq!(cfg.dir(), None);

        let opt1 = cfg.repos.get("use-package");
        assert_eq!(opt1.is_some(), true);
        let repo1 = opt1.unwrap();
        assert_eq!(repo1.url(), "https://github.com/jweigley/use-package.git");

        let opt2 = cfg.repos.get("magit");
        assert_eq!(opt2.is_some(), true);
        let repo2 = opt2.unwrap();
        assert_eq!(repo2.url(), "https://github.com/magit/magit.git");

        let opt3 = cfg.repos.get("forge");
        assert_eq!(opt3.is_some(), true);
        let repo3 = opt3.unwrap();
        assert_eq!(repo3.url(), "https://github.com/magit/forge.git");
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
            repos: IndexMap::new(),
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), None);
    }

    macro_rules! gh {
        ($p:expr, $n:expr) => (RepoKind::Default(Repo::GitHub(GitHub::new($p, $n))));
    }

    #[test]
    fn test_config_repos_iter_one() {
        let repo_kind = gh!("foo", "bar");
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), repo_kind.clone());
        let cfg = Config {
            dir: None,
            repos,
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", &repo_kind))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_multiple() {
        let repo_kind1 = gh!("foo1", "bar1");
        let repo_kind2 = gh!("foo2", "bar2");
        let repo_kind3 = gh!("foo3", "bar3");
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), repo_kind1.clone());
        repos.insert("two".to_string(), repo_kind2.clone());
        repos.insert("three".to_string(), repo_kind3.clone());
        let cfg = Config {
            dir: None,
            repos,
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", &repo_kind1))));
        assert_eq!(iter.next(), Some(Ok(("two", &repo_kind2))));
        assert_eq!(iter.next(), Some(Ok(("three", &repo_kind3))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_none_selected() {
        let cfg = Config {
            dir: None,
            repos: IndexMap::new(),
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
        let repo_kind1 = gh!("foo1", "bar1");
        let repo_kind2 = gh!("foo2", "bar2");
        let repo_kind3 = gh!("foo3", "bar3");
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), repo_kind1.clone());
        repos.insert("two".to_string(), repo_kind2.clone());
        repos.insert("three".to_string(), repo_kind3.clone());
        let cfg = Config {
            dir: None,
            repos,
        };

        let names = vec!["one", "three"];
        let mut iter = cfg.repos(Some(&names));
        assert_eq!(iter.next(), Some(Ok(("one", &repo_kind1))));
        assert_eq!(iter.next(), Some(Ok(("three", &repo_kind3))));
        assert_eq!(iter.next(), None);
    }
}
