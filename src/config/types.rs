use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct GitHub {
    pub user: String,
    pub project: String,
}

impl GitHub {
    pub fn new<S>(user: S, project: S) -> Self
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

#[derive(Clone, Debug, PartialEq)]
pub enum Selection<T> {
    Explicit(T),
    Optional(T),
}

impl<T> Selection<T> {
    pub fn repo(&self) -> &T {
        match self {
            Selection::Explicit(ref x) => x,
            Selection::Optional(ref x) => x,
        }
    }

    pub fn as_ref(&self) -> Selection<&T> {
        match self {
            Selection::Explicit(ref x) => Selection::Explicit(x),
            Selection::Optional(ref x) => Selection::Optional(x),
        }
    }
}

impl Remote for Selection<Repo> {
    fn url(&self) -> String {
        match self {
            Selection::Explicit(repo) => repo.url(),
            Selection::Optional(repo) => repo.url(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GitCmd {
    path: PathBuf,
}

impl GitCmd {
    pub fn new<P>(path: &P) -> Self where P: AsRef<Path> {
        GitCmd { path: path.as_ref().to_path_buf() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Default for GitCmd {
    fn default() -> Self {
        GitCmd { path: "git".into() }
    }
}
