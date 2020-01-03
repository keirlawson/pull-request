use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use rustygit::types::BranchName;
use std::str::FromStr;
use serde::Deserialize;
use structopt::StructOpt;
use std::io::prelude::*;
use pull_request::PullRequestOptions;
use pull_request::GithubRepository;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Deserialize)]
struct Config {
    repositories: Vec<GithubRepository>,
    branch_name: String,
    commit_mesage: String,
    pr_title: String
}

#[derive(StructOpt)]
struct Arguments {
    config: PathBuf,
    workspace: Option<PathBuf>
}
fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();

    let arguments = Arguments::from_args();

    let (options, repos) = read_config(&arguments.config);

    let github_token = env::var("GITHUB_TOKEN").unwrap();

    let transform = |p: &Path| {
        let epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        File::create(p.join(format!("{}", epoch))).unwrap();
        Ok(())
    };

    match pull_request::create_prs(&github_token, USER_AGENT, &options, transform, repos, arguments.workspace) {
        Ok(_) => println!("success"),
        Err(e) => eprintln!("{:?}", e),
    }
}

fn read_config(location: &Path) -> (PullRequestOptions, Vec<GithubRepository>) {
    let mut config_file = File::open(location).unwrap();

    
    let mut buffer = Vec::new();
    config_file.read_to_end(&mut buffer).unwrap();
    let config: Config = toml::de::from_slice(&buffer).unwrap();

    let options = pull_request::PullRequestOptions {
        branch_name: BranchName::from_str(&config.branch_name).unwrap(),
        commit_mesage: config.commit_mesage,
        pr_title: config.pr_title,
    };

    (options, config.repositories)
}
