use std::path::Path;

use ansi_term::Colour::Green;

use crate::config::{Config, Remote};

pub fn list(cfg: &Config) {
    for result in cfg.repos(None) {
        if let Ok((dir, repo)) = result {
            let path = Path::new(dir);
            println!(
                "{} {:<19}{}",
                if path.is_dir() {
                    Green.paint("o")
                } else {
                    " ".into()
                },
                dir,
                repo.url()
            );
        }
    }
}
