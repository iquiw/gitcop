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

