use envy;

fn main() {
    human_panic::setup_panic!();
    pretty_env_logger::init();

    pull_request::create_pr("", "", "", "", "").unwrap();
}
