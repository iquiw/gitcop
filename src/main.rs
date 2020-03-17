use std::env;
use std::process::exit;

use clap::{clap_app, crate_name, crate_version, AppSettings};

use gitcop::cmd;
use gitcop::config;
use gitcop::print;

#[tokio::main]
async fn main() {
    print::color_init();

    let matches = clap_app!(myapp =>
      (name: crate_name!())
      (version: crate_version!())
      (setting: AppSettings::ArgRequiredElseHelp)
      (setting: AppSettings::ColorAuto)
      (@subcommand list =>
        (about: "List repos")
        (@arg default: -d --default "List default repositories only")
        (@arg optional: -o --optional "List optional repositories only")
        (@arg unknown: -u --unknown "List unknown directories"))
      (@subcommand pull =>
        (about: "Pull in directories")
        (@arg DIR: +required ... "Directories to be pulled"))
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
            eprintln!(
                "Unable to change directory to \"{}\", {}",
                dir.display(),
                err
            );
            exit(1)
        }
    }
    match matches.subcommand() {
        ("list", Some(sub_m)) => {
            if sub_m.is_present("unknown") {
                cmd::list_unknown(&cfg)
            } else {
                let mut default = sub_m.is_present("default");
                let mut optional = sub_m.is_present("optional");
                if !default && !optional {
                    default = true;
                    optional = true;
                }
                cmd::list(&cfg, default, optional)
            }
        }
        ("pull", Some(sub_m)) => {
            if let Some(dirs) = sub_m.values_of("DIR") {
                cmd::pull(&cfg, dirs).await
            } else {
                Ok(())
            }
        }
        ("sync", Some(sub_m)) => {
            let names = sub_m.values_of("REPO").map(|vs| vs.collect());
            cmd::sync(&cfg, names.as_ref()).await
        }
        _ => Ok(()),
    }
    .unwrap_or_else(|err| {
        eprintln!(
            "gitcop: {} failed, Error: {}",
            matches.subcommand_name().unwrap_or("unknown"),
            err
        );
    })
}
