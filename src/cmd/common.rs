use std::sync::Arc;

use failure::Error;
use futures::future::{self, Future};
use futures::try_ready;
use tokio::prelude::Poll;
use tokio_sync::semaphore::{Permit, Semaphore};
use tokio_threadpool::SpawnHandle;

use crate::git::{AsyncGitResult, GitResult};
use crate::print;

pub struct BoundedProc<R> {
    semaphore: Arc<Semaphore>,
    proc: R,
    inner: Option<AsyncGitResult>,
    permit: Permit,
}

impl<R> BoundedProc<R> {
    pub fn new(proc: R, semaphore: Arc<Semaphore>) -> Self {
        BoundedProc {
            semaphore,
            proc,
            inner: None,
            permit: Permit::new(),
        }
    }
}

pub trait BoundedRun {
    fn run(&self) -> AsyncGitResult;
}

impl<R> Future for BoundedProc<R>
where
    R: BoundedRun,
{
    type Item = GitResult;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if !self.permit.is_acquired() {
            try_ready!(self.permit.poll_acquire(&self.semaphore));
        }
        let ready = match self.inner {
            Some(ref mut inner) => inner.poll()?,
            None => {
                let mut inner = self.proc.run();
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

pub fn join_handles(name: &str, handles: Vec<SpawnHandle<GitResult, Error>>) -> Result<(), Error> {
    future::join_all(handles)
        .map(|results| {
            let mut has_error = false;
            for result in results {
                if let GitResult::Error(key, msg) = result {
                    if !has_error {
                        println!("\nThe following {} got error!", name);
                        has_error = true;
                    }
                    println!("{}: {}", print::warn(&key), msg);
                }
            }
        })
        .wait()
}
