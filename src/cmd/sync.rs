use std::io::stdout;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ansi_term::Colour::Red;
use failure::Error;
use futures::future::{self, Future};
use tokio;
use tokio::prelude::*;
use tokio_sync::semaphore::Semaphore;
use tokio_threadpool::Builder;

use super::common::{BoundedProc, BoundedRun};
use crate::config::{Config, Repo, Selection};
use crate::git::{AsyncGitResult, Git, GitCmd, GitResult};

struct BoundedSync {
    dir: PathBuf,
    repo: Repo,
}

impl BoundedRun for BoundedSync {
    fn run(&self) -> AsyncGitResult {
        let git = GitCmd::default();
        sync_one(&self.dir, &self.repo, &git)
    }
}

pub fn sync(cfg: &Config, names: Option<&Vec<&str>>) -> Result<(), Error> {
    let pool = Builder::new().build();
    let sem = Arc::new(Semaphore::new(10));
    let mut handles = vec![];
    for result in cfg.repos(names) {
        match result {
            Ok((dir, select)) => {
                let repo = match select {
                    Selection::Explicit(repo) => repo,
                    Selection::Optional(repo) => {
                        if Path::new(dir).is_dir() {
                            repo
                        } else {
                            continue;
                        }
                    }
                };
                let sem = Arc::clone(&sem);
                let path = Path::new(&dir);
                handles.push(pool.spawn_handle(BoundedProc::new(
                    BoundedSync {
                        dir: path.to_path_buf(),
                        repo: repo.clone(),
                    },
                    sem,
                )));
            }
            Err(err) => {
                let stdout = stdout();
                let mut handle = stdout.lock();
                writeln!(&mut handle, "{}", err).unwrap();
            }
        }
    }
    pool.shutdown_on_idle().wait().unwrap();
    future::join_all(handles)
        .map(|results| {
            let mut has_error = false;
            for result in results {
                if let GitResult::Error(key, msg) = result {
                    if !has_error {
                        println!("\nThe following sync got error!");
                        has_error = true;
                    }
                    println!("{}: {}", Red.paint(&key), msg);
                }
            }
        })
        .wait()
        .unwrap();
    Ok(())
}

pub fn sync_one(dir: &Path, repo: &Repo, git: &Git) -> AsyncGitResult {
    if dir.is_dir() {
        git.pull(dir)
    } else {
        git.cloner(dir, &repo)
    }
}
