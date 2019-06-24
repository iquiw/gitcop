use std::path::Path;

use ansi_term::Colour::Green;

use crate::config::{Config, Remote, RepoKind};

pub fn list(cfg: &Config) {
    for result in cfg.repos(None) {
        if let Ok((dir, repo_kind)) = result {
            let exist = Path::new(dir).is_dir();
            let (mark, repo) = match repo_kind {
                RepoKind::Default(repo) => (if exist { "*" } else { "-" }, repo),
                RepoKind::Optional(repo) => (if exist { "o" } else { " " }, repo),
            };
            println!("{} {:<19} {}", Green.paint(mark), dir, repo.url());
        }
    }
}
