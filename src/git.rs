use std::fmt;
use std::future::Future;
use std::io;
use std::path::Path;
use std::process::Output;

use anyhow::Error;
use futures::future::BoxFuture;
use tokio::process::Command;

use crate::config::{GitCmd, Remote, Repo};
use crate::print;

pub trait Git<'a> {
    fn cloner(&'a self, dir: &Path, repo: &Repo) -> AsyncGitResult<'a>;
    fn pull(&'a self, dir: &Path) -> AsyncGitResult<'a>;
}

pub type GitResult = Result<String, Error>;
pub type AsyncGitResult<'a> = BoxFuture<'a, GitResult>;

#[derive(Debug)]
pub struct GitError {
    pub key: String,
    pub msg: String,
}

impl std::error::Error for GitError {}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.msg)
    }
}

impl<'a> Git<'a> for GitCmd {
    fn cloner(&'a self, dir: &Path, repo: &Repo) -> AsyncGitResult<'a> {
        let future = Command::new(self.path())
            .arg("-c")
            .arg("color.ui=always")
            .arg("clone")
            .arg(repo.url())
            .arg(dir)
            .output();
        let key = dir.to_string_lossy().into_owned();
        Box::pin(process_output(key, future))
    }

    fn pull(&'a self, dir: &Path) -> AsyncGitResult<'a> {
        let future = Command::new(self.path())
            .current_dir(dir)
            .arg("-c")
            .arg("color.ui=always")
            .arg("pull")
            .arg("--ff-only")
            .output();
        let key = dir.to_string_lossy().into_owned();
        Box::pin(process_output(key, future))
    }
}

async fn process_output<F>(key: String, out: F) -> Result<String, Error>
where
    F: Future<Output = Result<Output, io::Error>> + Send,
{
    let output = out.await?;
    let success = output.status.success();
    let colorize = if success { print::good } else { print::warn };
    print!(
        "[{}] {}{}",
        colorize(&key),
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?
    );

    if output.status.success() {
        Ok(key)
    } else {
        Err(GitError {
            key,
            msg: format!("{}", output.status),
        }.into())
    }
}
