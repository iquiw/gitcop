use std::future::Future;
use std::sync::Arc;

use failure::Error;
use futures::future;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

use crate::git::GitError;
use crate::print;

pub async fn bounded_run<R>(run: R, semaphore: Arc<Semaphore>) -> R::Output
where
    R: Future,
{
    let permit = semaphore.acquire().await;
    let result = run.await;
    drop(permit);
    result
}

pub async fn join_handles(
    name: &str,
    handles: Vec<JoinHandle<Result<String, Error>>>,
) -> Result<(), Error> {
    let results = future::join_all(handles).await;
    let mut has_error = false;
    for result in results {
        if let Err(err) = result? {
            let git_err = err.downcast::<GitError>()?;
            if !has_error {
                println!("\nThe following {} got error!", name);
                has_error = true;
            }
            println!("{}: {}", print::warn(&git_err.key), git_err.msg);
        }
    }
    Ok(())
}
