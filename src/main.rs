use std::env;
use std::process::exit;

use clap::{crate_name, crate_version, Arg, ArgAction, Command};

use gitcop::cmd;
use gitcop::config;
use gitcop::print;

#[tokio::main]
async fn main() {
    print::color_init();

    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .arg_required_else_help(true)
        .subcommands([
            Command::new("list")
                .about("List repos")
                .arg(
                    Arg::new("default")
                        .short('d')
                        .long("default")
                        .action(ArgAction::SetTrue)
                        .help("List default repositories only"),
                )
                .arg(
                    Arg::new("optional")
                        .short('o')
                        .long("optional")
                        .action(ArgAction::SetTrue)
                        .help("List optional repositories only"),
                )
                .arg(
                    Arg::new("unknown")
                        .short('u')
                        .long("unknown")
                        .action(ArgAction::SetTrue)
                        .help("List unknown directories"),
                ),
            Command::new("pull").about("Pull in directories").arg(
                Arg::new("DIR")
                    .required(true)
                    .action(ArgAction::Append)
                    .num_args(1..),
            ),
            Command::new("sync")
                .about("Sync repos")
                .arg(Arg::new("REPO").action(ArgAction::Append).num_args(0..)),
        ])
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
        Some(("list", sub_m)) => {
            if sub_m.get_flag("unknown") {
                cmd::list_unknown(&cfg)
            } else {
                let mut default = sub_m.get_flag("default");
                let mut optional = sub_m.get_flag("optional");
                if !default && !optional {
                    default = true;
                    optional = true;
                }
                cmd::list(&cfg, default, optional)
            }
        }
        Some(("pull", sub_m)) => {
            if let Some(dirs) = sub_m.get_many::<String>("DIR") {
                cmd::pull(&cfg, dirs.map(|s| s.as_str())).await
            } else {
                Ok(())
            }
        }
        Some(("sync", sub_m)) => {
            if let Some(names) = sub_m.get_many::<String>("REPO") {
                cmd::sync(&cfg, Some(&names.map(|s| s.as_str()).collect())).await
            } else {
                cmd::sync(&cfg, None).await
            }
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
