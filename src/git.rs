use std::path::Path;
use std::process::Command;

use failure::{format_err, Error};
use futures::Future;
use tokio_process::{CommandExt, OutputAsync};

use crate::config::{GitCmd, Remote, Repo};
use crate::locked_print;
use crate::print;

pub trait Git {
    fn cloner(&self, dir: &Path, repo: &Repo) -> AsyncGitResult;
    fn pull(&self, dir: &Path) -> AsyncGitResult;
}

pub type AsyncGitResult = Box<dyn Future<Item = GitResult, Error = Error> + Send>;

pub enum GitResult {
    Success(String),
    Error(String, Error),
}

impl Git for GitCmd {
    fn cloner(&self, dir: &Path, repo: &Repo) -> AsyncGitResult {
        let future = Command::new(self.path())
            .arg("-c")
            .arg("color.ui=always")
            .arg("clone")
            .arg(repo.url())
            .arg(dir)
            .output_async();
        process_output(dir, future)
    }

    fn pull(&self, dir: &Path) -> AsyncGitResult {
        let future = Command::new(self.path())
            .current_dir(dir)
            .arg("-c")
            .arg("color.ui=always")
            .arg("pull")
            .arg("--ff-only")
            .output_async();
        process_output(dir, future)
    }
}

fn process_output(dir: &Path, out: OutputAsync) -> AsyncGitResult {
    let key = dir.to_string_lossy().into_owned();
    let future = out.map_err(|e| e.into()).and_then(|output| {
        let success = output.status.success();
        let colorize = if success { print::good } else { print::warn };
        locked_print!(
            "[{}] {}{}",
            colorize(&key),
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        );

        if output.status.success() {
            Ok(GitResult::Success(key))
        } else {
            Ok(GitResult::Error(key, format_err!("{}", output.status)))
        }
    });
    Box::new(future)
}
