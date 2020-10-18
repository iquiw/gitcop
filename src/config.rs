use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::slice;

use failure::{Error, Fail};
use indexmap::{self, IndexMap};

mod internal;
mod types;
use self::internal::{Concurrency, ConfigInternal};
pub use self::types::{GitCmd, GitHub, Remote, Repo, Selection};
use crate::print;

#[derive(Debug)]
pub struct Config {
    git: GitCmd,
    dir: Option<PathBuf>,
    concur: Concurrency,
    repos: IndexMap<String, Selection<Repo>>,
}

impl Config {
    pub fn git(&self) -> &GitCmd {
        &self.git
    }

    pub fn dir(&self) -> Option<&PathBuf> {
        self.dir.as_ref()
    }

    pub fn concurrency(&self) -> usize {
        self.concur.value() as usize
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.repos.contains_key(name)
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
    iter: indexmap::map::Iter<'a, String, Selection<Repo>>,
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
        write!(f, "{}: Repo not found", print::warn(&self.name))
    }
}

impl<'a> Iterator for ReposIter<'a> {
    type Item = Result<(&'a str, Selection<&'a Repo>), RepoNotFound>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ReposIter::Selected(repo_sel) => {
                if let Some(n) = repo_sel.names.next() {
                    if let Some(sel) = repo_sel.cfg.repos.get(*n) {
                        if let Selection::Optional(repo) = sel {
                            return Some(Ok((n, Selection::Explicit(repo))));
                        } else {
                            return Some(Ok((n, sel.as_ref())));
                        }
                    } else {
                        return Some(Err(RepoNotFound {
                            name: n.to_string(),
                        }));
                    }
                }
                None
            }
            ReposIter::All(ReposAll { iter }) => {
                iter.next().map(|(s, repo)| Ok((s.as_ref(), repo.as_ref())))
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
    let git = cfgi.git;
    let dir = cfgi.directory;
    let mut repo_map = IndexMap::new();
    for (key, val) in &cfgi.repositories {
        let repo = Repo::try_from((key.as_str(), val))?;
        repo_map.insert(key.to_string(), Selection::Explicit(repo));
    }
    if let Some(opt_repos) = &cfgi.optional_repositories {
        for (key, val) in opt_repos {
            let repo = Repo::try_from((key.as_str(), val))?;
            repo_map.insert(key.to_string(), Selection::Optional(repo));
        }
    }
    Ok(Config {
        git,
        dir: dir.map(PathBuf::from),
        concur: cfgi.concurrency,
        repos: repo_map,
    })
}

#[cfg(test)]
mod test {
    use crate::config::internal::Concurrency;
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

        assert_eq!(cfg.git(), &GitCmd::default());
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
    fn test_parse_config_with_custom_git_dir_concur() {
        let s = r#"git = "/opt/bin/git"
directory = "/tmp/foo"
concurrency = 123
[repositories]
"#;
        let cfg = parse_config(s).unwrap();

        assert_eq!(cfg.git(), &GitCmd::new(&Path::new("/opt/bin/git")));
        assert_eq!(cfg.dir(), Some(&PathBuf::from("/tmp/foo")));
        assert_eq!(cfg.concurrency(), 123);
    }

    #[test]
    fn test_parse_config_with_invalid_concur() {
        let result = parse_config("concurrency = -1\n[repositories]");
        assert_eq!(result.is_err(), true);

        let result = parse_config("concurrency = 0\n[repositories]");
        assert_eq!(result.is_err(), true);

        let result = parse_config("concurrency = 65536\n[repositories]");
        assert_eq!(result.is_err(), true);

        let result = parse_config("concurrency = NaN\n[repositories]");
        assert_eq!(result.is_err(), true);
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
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
            repos: IndexMap::new(),
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), None);
    }

    macro_rules! gh {
        ($p:expr, $n:expr) => {
            Selection::Explicit(Repo::GitHub(GitHub::new($p, $n)))
        };
        ($p:expr, $n:expr, o) => {
            Selection::Optional(Repo::GitHub(GitHub::new($p, $n)))
        };
    }

    #[test]
    fn test_config_repos_iter_one() {
        let select = gh!("foo", "bar");
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), select.clone());
        let cfg = Config {
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
            repos,
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", select.as_ref()))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_multiple() {
        let select1 = gh!("foo1", "bar1");
        let select2 = gh!("foo2", "bar2");
        let select3 = gh!("foo3", "bar3");
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), select1.clone());
        repos.insert("two".to_string(), select2.clone());
        repos.insert("three".to_string(), select3.clone());
        let cfg = Config {
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
            repos,
        };
        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", select1.as_ref()))));
        assert_eq!(iter.next(), Some(Ok(("two", select2.as_ref()))));
        assert_eq!(iter.next(), Some(Ok(("three", select3.as_ref()))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_none_selected() {
        let cfg = Config {
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
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
        let select1 = gh!("foo1", "bar1");
        let select2 = gh!("foo2", "bar2");
        let select3 = gh!("foo3", "bar3");
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), select1.clone());
        repos.insert("two".to_string(), select2.clone());
        repos.insert("three".to_string(), select3.clone());
        let cfg = Config {
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
            repos,
        };

        let names = vec!["one", "three"];
        let mut iter = cfg.repos(Some(&names));
        assert_eq!(iter.next(), Some(Ok(("one", select1.as_ref()))));
        assert_eq!(iter.next(), Some(Ok(("three", select3.as_ref()))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_optional_none_selected() {
        let select1 = gh!("foo1", "bar1", o);
        let select2 = gh!("foo2", "bar2", o);
        let select3 = gh!("foo3", "bar3", o);
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), select1.clone());
        repos.insert("two".to_string(), select2.clone());
        repos.insert("three".to_string(), select3.clone());
        let cfg = Config {
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
            repos,
        };

        let mut iter = cfg.repos(None);
        assert_eq!(iter.next(), Some(Ok(("one", select1.as_ref()))));
        assert_eq!(iter.next(), Some(Ok(("two", select2.as_ref()))));
        assert_eq!(iter.next(), Some(Ok(("three", select3.as_ref()))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_config_repos_iter_optional_multiple_selected() {
        let select1 = gh!("foo1", "bar1", o);
        let select2 = gh!("foo2", "bar2", o);
        let select3 = gh!("foo3", "bar3", o);
        let mut repos = IndexMap::new();
        repos.insert("one".to_string(), select1.clone());
        repos.insert("two".to_string(), select2.clone());
        repos.insert("three".to_string(), select3.clone());
        let cfg = Config {
            git: GitCmd::default(),
            dir: None,
            concur: Concurrency::default(),
            repos,
        };

        let names = vec!["two", "three"];
        let mut iter = cfg.repos(Some(&names));
        // changed to RepoKind::Selected
        assert_eq!(iter.next(), Some(Ok(("two", gh!("foo2", "bar2").as_ref()))));
        assert_eq!(
            iter.next(),
            Some(Ok(("three", gh!("foo3", "bar3").as_ref())))
        );
        assert_eq!(iter.next(), None);
    }
}
