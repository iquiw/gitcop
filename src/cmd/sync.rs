use std::path::{Path, PathBuf};
use std::sync::Arc;

use failure::Error;
use futures::future::Future;
use tokio_sync::semaphore::Semaphore;
use tokio_threadpool::Builder;

use super::common::{join_handles, BoundedProc, BoundedRun};
use crate::config::{Config, GitCmd, Repo, Selection};
use crate::git::{AsyncGitResult, Git};
use crate::locked_println;

struct BoundedSync {
    git: GitCmd,
    dir: PathBuf,
    repo: Repo,
}

impl BoundedRun for BoundedSync {
    fn run(&self) -> AsyncGitResult {
        if self.dir.is_dir() {
            self.git.pull(&self.dir)
        } else {
            self.git.cloner(&self.dir, &self.repo)
        }
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
                        git: cfg.git().clone(),
                        dir: path.to_path_buf(),
                        repo: repo.clone(),
                    },
                    sem,
                )));
            }
            Err(err) => {
                locked_println!("{}", err);
            }
        }
    }
    pool.shutdown_on_idle().wait().unwrap();
    join_handles("sync", handles)
}
