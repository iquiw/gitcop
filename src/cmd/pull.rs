use std::path::PathBuf;
use std::sync::Arc;

use failure::Error;
use tokio::sync::Semaphore;

use super::common::{bounded_run, join_handles};
use crate::config::Config;
use crate::git::Git;
use crate::print;

pub async fn pull<'a, I>(cfg: &Config, dirs: I) -> Result<(), Error>
where
    I: Iterator<Item = &'a str>,
{
    let sem = Arc::new(Semaphore::new(cfg.concurrency()));
    let mut handles = vec![];
    for dir in dirs {
        let sem = Arc::clone(&sem);
        let path = PathBuf::from(dir);
        if !path.is_dir() {
            println!("{}: No such directory", print::warn(dir));
            continue;
        }
        let mut git_path = path.clone();
        git_path.push(".git");
        if !git_path.exists() {
            println!("{}: Not git repository", print::warn(dir));
            continue;
        }
        let git = cfg.git().clone();
        let path = PathBuf::from(&dir);
        handles.push(tokio::spawn(async move {
            bounded_run(git.pull(&path), sem).await
        }));
    }
    join_handles("pull", handles).await
}
