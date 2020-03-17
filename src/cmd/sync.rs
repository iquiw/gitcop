use std::path::{Path, PathBuf};
use std::sync::Arc;

use failure::Error;
use tokio::sync::Semaphore;

use super::common::{bounded_run, join_handles};
use crate::config::{Config, Repo, Selection};
use crate::git::{Git, GitResult};

async fn sync_one<'a, G>(git: &'a G, dir: &Path, repo: &Repo) -> GitResult
where
    G: Git<'a>,
{
    if dir.is_dir() {
        git.pull(&dir).await
    } else {
        git.cloner(&dir, &repo).await
    }
}

pub async fn sync(cfg: &Config, names: Option<&Vec<&str>>) -> Result<(), Error> {
    let sem = Arc::new(Semaphore::new(10));
    let mut handles = vec![];
    for result in cfg.repos(names) {
        match result {
            Ok((dir, select)) => {
                let repo = match select {
                    Selection::Explicit(repo) => repo.clone(),
                    Selection::Optional(repo) => {
                        if Path::new(dir).is_dir() {
                            repo.clone()
                        } else {
                            continue;
                        }
                    }
                };
                let sem = Arc::clone(&sem);
                let path = PathBuf::from(&dir);
                let git = cfg.git().clone();
                handles.push(tokio::spawn(async move {
                    bounded_run(sync_one(&git, &path, &repo), sem).await
                }));
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
    join_handles("sync", handles).await
}
