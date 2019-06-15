use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use ansi_term::Colour::{Green, Red};
use failure::{err_msg, Error};
use futures::Future;
use tokio_process::{CommandExt, OutputAsync};

use crate::config::{Remote, Repo};

pub trait Git {
    fn cloner(&self, dir: &Path, repo: &Repo) -> AsyncGitResult;
    fn pull(&self, dir: &Path) -> AsyncGitResult;
}

pub type AsyncGitResult = Box<Future<Item = GitResult, Error = Error> + Send>;

pub enum GitResult {
    Success(String),
    Error(String, Error),
}

pub struct GitCmd {
    path: PathBuf,
}

impl Default for GitCmd {
    fn default() -> Self {
        GitCmd { path: "git".into() }
    }
}

impl Git for GitCmd {
    fn cloner(&self, dir: &Path, repo: &Repo) -> AsyncGitResult {
        let future = Command::new(&self.path)
            .arg("clone")
            .arg(repo.url())
            .arg(dir)
            .output_async();
        process_output(dir, future)
    }

    fn pull(&self, dir: &Path) -> AsyncGitResult {
        let future = Command::new(&self.path)
            .current_dir(dir)
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
        let color = if success { Green } else { Red };
        let stdout = stdout();
        let mut handle = stdout.lock();
        write!(
            &mut handle,
            "[{}] {}{}",
            color.paint(&key),
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        )
        .unwrap();

        if output.status.success() {
            Ok(GitResult::Success(key))
        } else {
            Ok(GitResult::Error(key, err_msg(format!("{}", output.status))))
        }
    });
    Box::new(future)
}
