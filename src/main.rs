use std::env;
use std::process::exit;

use clap::{clap_app, crate_name, crate_version};

use gitcop::cmd;
use gitcop::config;

fn main() {
    let matches = clap_app!( myapp =>
      (name: crate_name!())
      (version: crate_version!())
      (@subcommand sync =>
        (about: "Sync repos")
        (@arg REPO: ... "Name of repos"))
    )
    .get_matches();

    let cfg = match config::load_config(".gitcop.toml") {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Unable to load .gitcop.toml, {}", err);
            exit(1)
        }
    };
    if let Some(dir) = cfg.dir() {
        if let Err(err) = env::set_current_dir(dir) {
            eprintln!("Unable to change directory to \"{}\", {}", dir.display(), err);
            exit(1)
        }
    }
    match matches.subcommand() {
        ("sync", Some(sub_m)) => {
            let names = sub_m
                .values_of("REPO")
                .map(|vs| vs.collect())
                .unwrap_or(vec![]);
            if let Err(err) = cmd::sync(&cfg, &names) {
                eprintln!("gitcop: sync failed, Error: {}", err);
            }
        }
        _ => eprintln!("{}", matches.usage()),
    }
}
