use std::env;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;
use rustygit::types::BranchName;
use std::str::FromStr;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();
    let github_token = env::var("GITHUB_TOKEN").unwrap();

    let options = pull_request::PullRequestOptions {
        organisation: "RustyGitTestOrg",
        repository: "ForkMe",
        branch_name: BranchName::from_str("thebranch3").unwrap(),
        commit_mesage: "test commit",
        pr_title: "test PR",
    };

    let transform = |p: &Path| {
        let epoch = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        File::create(p.join(format!("{}", epoch))).unwrap();
        Ok(())
    };

    match pull_request::create_pr(&github_token, USER_AGENT, &options, transform) {
        Ok(_) => println!("success"),
        Err(e) => eprintln!("{:?}", e),
    }
}
