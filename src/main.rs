use std::env;
use std::path::Path;
fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();
    let github_token = env::var("GITHUB_TOKEN").unwrap();

    let options = pull_request::PullRequestOptions {
        organisation: "RustyGitTestOrg",
        repository: "ForkMe",
        branch_name: "thebranch1",
        commit_mesage: "test commit",
        pr_title: "test PR",
    };

    //FIXME do something here
    let transform = |p : &Path| Ok(());

    match pull_request::create_pr(&github_token, "my-cool-user-agent/0.1.0", &options, transform) {
        Ok(_) => println!("success"),
        Err(e) => eprintln!("{:?}", e)
    }
}
