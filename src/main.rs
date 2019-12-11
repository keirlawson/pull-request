use std::env;
use std::path::Path;
use std::fs::File;
use std::time::SystemTime;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();
    let github_token = env::var("GITHUB_TOKEN").unwrap();

    let options = pull_request::PullRequestOptions {
        organisation: "RustyGitTestOrg",
        repository: "ForkMe",
        branch_name: "thebranch3",
        commit_mesage: "test commit",
        pr_title: "test PR",
    };

    let transform = |p : &Path| {
        let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        File::create(p.join(format!("{}", epoch))).unwrap(); 
        Ok(())
    };

    match pull_request::create_pr(&github_token, USER_AGENT, &options, transform) {
        Ok(_) => println!("success"),
        Err(e) => eprintln!("{:?}", e)
    }
}
