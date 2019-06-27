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
