use std::path::{Path, PathBuf};
use std::sync::Arc;

use failure::Error;
use futures::future::{self, Future};
use futures::try_ready;
use tokio;
use tokio::prelude::*;
use tokio_sync::semaphore::{Permit, Semaphore};
use tokio_threadpool::Builder;

use crate::config::{Config, Repo};
use crate::git::{AsyncGitResult, Git, GitCmd, GitResult};

struct BoundedSync {
    semaphore: Arc<Semaphore>,
    dir: PathBuf,
    repo: Repo,
    inner: Option<AsyncGitResult>,
    permit: Permit,
}

impl BoundedSync {
    fn new(dir: &Path, repo: Repo, semaphore: Arc<Semaphore>) -> Self {
        BoundedSync {
            semaphore,
            dir: dir.to_path_buf(),
            repo,
            inner: None,
            permit: Permit::new(),
        }
    }
}

impl Future for BoundedSync {
    type Item = GitResult;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if !self.permit.is_acquired() {
            try_ready!(self.permit.poll_acquire(&self.semaphore));
        }
        let ready = match self.inner {
            Some(ref mut inner) => inner.poll()?,
            None => {
                let git = GitCmd::default();
                let mut inner = sync_one(&self.dir, &self.repo, &git);
                let ready = inner.poll()?;
                self.inner = Some(inner);
                ready
            }
        };
        if ready.is_ready() {
            self.permit.release(&self.semaphore);
        }
        Ok(ready)
    }
}

pub fn sync(cfg: &Config) -> Result<(), Error> {
    let pool = Builder::new().build();
    let sem = Arc::new(Semaphore::new(5));
    let mut handles = vec![];
    for (dir, repo) in &cfg.repos {
        let sem = Arc::clone(&sem);
        let path = Path::new(dir);
        handles.push(pool.spawn_handle(BoundedSync::new(&path, repo.clone(), sem)));
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
                    println!("{}: {}", key, msg);
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
