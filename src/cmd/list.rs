use std::fs;
use std::path::Path;

use ansi_term::Colour::Green;

use crate::config::{Config, Remote, Selection};

pub fn list(cfg: &Config) {
    for result in cfg.repos(None) {
        if let Ok((dir, select)) = result {
            let exist = Path::new(dir).is_dir();
            let (mark, repo) = match select {
                Selection::Explicit(repo) => (if exist { "*" } else { "-" }, repo),
                Selection::Optional(repo) => (if exist { "o" } else { " " }, repo),
            };
            println!("{} {:<19} {}", Green.paint(mark), dir, repo.url());
        }
    }
}

pub fn list_unknown(cfg: &Config) {
    match fs::read_dir(".") {
        Ok(rdir) => {
            for result in rdir {
                if let Ok(entry) = result {
                    if !entry.path().is_dir() {
                        continue;
                    }
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy();
                    if !cfg.is_known(&name) && !name.starts_with(".") {
                        println!("{}", name);
                    }
                }
            }
        }
        Err(err) => eprintln!("Unable to read directory: {}", err),
    }
}
