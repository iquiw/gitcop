use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ansi_term::Colour::Red;
use failure::Error;
use futures::future::{self, Future};
use tokio_sync::semaphore::Semaphore;
use tokio_threadpool::Builder;

use super::common::{BoundedProc, BoundedRun};
use crate::git::{AsyncGitResult, Git, GitCmd, GitResult};

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
        let path = Path::new(dir);
        if !path.is_dir() {
            let stdout = stdout();
            let mut handle = stdout.lock();
            writeln!(&mut handle, "{}: No such directory", Red.paint(dir)).unwrap();
            continue;
        }
        handles.push(pool.spawn_handle(BoundedProc::new(
            BoundedPull {
                dir: path.to_path_buf(),
            },
            sem,
        )));
    }
    pool.shutdown_on_idle().wait().unwrap();
    future::join_all(handles)
        .map(|results| {
            let mut has_error = false;
            for result in results {
                if let GitResult::Error(key, msg) = result {
                    if !has_error {
                        println!("\nThe following pull got error!");
                        has_error = true;
                    }
                    println!("{}: {}", Red.paint(&key), msg);
                }
            }
        })
        .wait()
}
