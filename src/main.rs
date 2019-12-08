use std::env;
fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();
    let github_token = env::var("GITHUB_TOKEN").unwrap();

    match pull_request::create_pr("RustyGitTestOrg", "ForkMe", "thebranch1", "test commit", "", &github_token) {
        Ok(_) => println!("success"),
        Err(e) => eprintln!("{:?}", e)
    }
}
