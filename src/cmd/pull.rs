use std::path::PathBuf;
use std::sync::Arc;

use failure::Error;
use futures::future::Future;
use tokio_sync::semaphore::Semaphore;
use tokio_threadpool::Builder;

use super::common::{join_handles, BoundedProc, BoundedRun};
use crate::git::{AsyncGitResult, Git, GitCmd};
use crate::locked_println;
use crate::print;

struct BoundedPull {
    dir: PathBuf,
}

impl BoundedRun for BoundedPull {
    fn run(&self) -> AsyncGitResult {
        let git = GitCmd::default();
        git.pull(&self.dir)
    }
}

pub fn pull<'a, I>(dirs: I) -> Result<(), Error>
where
    I: Iterator<Item = &'a str>,
{
    let pool = Builder::new().build();
    let sem = Arc::new(Semaphore::new(10));
    let mut handles = vec![];
    for dir in dirs {
        let sem = Arc::clone(&sem);
        let path = PathBuf::from(dir);
        if !path.is_dir() {
            locked_println!("{}: No such directory", print::warn(dir));
            continue;
        }
        let mut git_path = path.clone();
        git_path.push(".git");
        if !git_path.exists() {
            locked_println!("{}: Not git repository", print::warn(dir));
            continue;
        }
        handles.push(pool.spawn_handle(BoundedProc::new(BoundedPull { dir: path }, sem)));
    }
    pool.shutdown_on_idle().wait().unwrap();
    join_handles("pull", handles)
}
