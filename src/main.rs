use clap::{App, Arg};
use git_remote_swap::RemoteSwapConfig;
use serde::Deserialize;

#[allow(unused_imports)]
use std::collections::HashMap;

use std::{io, path};

static DEFAULT_CONFIG_PATH: &str = "git-remote-swap.yaml";
static DEFAULT_ROOT_PATH: &str = ".";

fn main() -> io::Result<()> {
    let matches = register_clap();
    let mut app_config_raw = config::Config::default();

    let app_config_path = matches
        .value_of("config_path")
        .unwrap_or(DEFAULT_CONFIG_PATH);

    let root_path = path::Path::new(matches.value_of("root_path").unwrap_or(DEFAULT_ROOT_PATH));
    let dry_run = matches.occurrences_of("dry_run") > 0;

    app_config_raw
        .merge(config::File::from(path::Path::new(app_config_path)))
        .unwrap();

    let app_config = app_config_raw.try_into::<AppConfigFile>().unwrap();

    let remote_mapping = app_config
        .remotes
        .into_iter()
        .map(|r| (r.replace, r.with))
        .collect();

    git_remote_swap::run(RemoteSwapConfig::new(
        dry_run,
        root_path.to_path_buf(),
        remote_mapping,
    ))
    .unwrap();

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AppConfigFile {
    remotes: Vec<AppConfigRemoteEntry>,
}

#[derive(Debug, Deserialize)]
struct AppConfigRemoteEntry {
    replace: String,
    with: String,
}

fn register_clap<'a>() -> clap::ArgMatches<'a> {
    App::new("git-remote-swap")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Chad Gilbert <chad@freakingawesome.net>")
        .about("Useful when you have a large number of repositories have been migrated to a different remote. This scours your filesystem for repos with retired remotes and points them at the new URL.")
        .arg(Arg::with_name("config_path")
            .short("c")
            .long("config")
            .takes_value(true)
            .value_name("PATH")
            .help("The path to the configuration file")
            .default_value(DEFAULT_CONFIG_PATH))
        .arg(Arg::with_name("root_path")
            .short("r")
            .long("root")
            .takes_value(true)
            .value_name("PATH")
            .help("The root directory where the crawl should start")
            .default_value(DEFAULT_ROOT_PATH))
        .arg(Arg::with_name("dry_run")
            .long("dry-run")
            .takes_value(false)
            .help("Logs output but does not swap any remotes"))
        .get_matches()
}
