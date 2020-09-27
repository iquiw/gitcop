use std::fs;
use std::path::Path;

use anyhow::{anyhow, Error};

use crate::config::{Config, Remote, Selection};
use crate::print;

pub fn list(cfg: &Config, default: bool, optional: bool) -> Result<(), Error> {
    for result in cfg.repos(None) {
        if let Ok((dir, select)) = result {
            let exist = Path::new(dir).is_dir();
            let (mark, repo) = match select {
                Selection::Explicit(repo) => {
                    if !default {
                        continue;
                    } else {
                        (if exist { "*" } else { "-" }, repo)
                    }
                }
                Selection::Optional(repo) => {
                    if !optional {
                        continue;
                    } else {
                        (if exist { "o" } else { " " }, repo)
                    }
                }
            };
            println!("{} {:<19} {}", print::good(mark), dir, repo.url());
        }
    }
    Ok(())
}

pub fn list_unknown(cfg: &Config) -> Result<(), Error> {
    let rdir = fs::read_dir(".").map_err(|e| anyhow!("Unable to read directory, {}", e))?;
    for result in rdir {
        if let Ok(entry) = result {
            if !entry.path().is_dir() {
                continue;
            }
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if !cfg.is_known(&name) && !name.starts_with('.') {
                println!("{}", name);
            }
        }
    }
    Ok(())
}
