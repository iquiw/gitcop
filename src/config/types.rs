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
