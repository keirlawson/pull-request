use std::env;
fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();
    let github_token = env::var("GITHUB_TOKEN").unwrap();

    pull_request::create_pr("", "", "", "", "", &github_token).unwrap();
}
