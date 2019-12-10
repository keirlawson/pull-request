use rustygit::{types::GitUrl, Repository};
use std::str::FromStr;
use tempfile;
use url::Url;
use log::debug;

use hubcaps::Result;

mod github;

const DEFAULT_UPSTREAM_REMOTE: &str = "upstream";

pub struct PullRequestOptions<'a> {
    pub organisation: &'a str,
    pub repository: &'a str,
    pub branch_name: &'a str,
    pub commit_mesage: &'a str,
    pub pr_title: &'a str,
}

pub fn create_pr(github_token: &str, options: PullRequestOptions) -> Result<Url> {
    //FIXME validate that strings are not empty

    //FIXME pass in user agent
    let mut github_client =
        github::GithubClient::init("my-cool-user-agent/0.1.0", github_token)?;

    let username = github_client.get_username()?;
    debug!("Retrieved username for github account: {}", username);

    let fork = github_client.existing_fork(username.as_str(), options.organisation, options.repository)?;

    let fork = if let Some(existing) = fork {
        existing
    } else {
        debug!("No fork exists, forking");
        github_client.create_fork(options.organisation, options.repository)?
    };

    //FIXME allow users to specify path
    let tmp_dir = tempfile::tempdir()?;

    //FIXME allow user to specify SSH or HTTPS
    let url = GitUrl::from_str(&fork.ssh_url).expect("github returned malformed clone URL");
    debug!("Cloning repo to {:?}", tmp_dir.path());

    let repo = Repository::clone(url, tmp_dir.path()).unwrap();

    //FIXME check if upstream remote exists

    //FIXME support ssh URLs as well, what about custom githubs?
    let upstream = GitUrl::from_str(format!("https://github.com/{}/{}.git", options.organisation, options.repository).as_str()).unwrap();
    repo.add_remote(DEFAULT_UPSTREAM_REMOTE, &upstream).unwrap();

    repo.fetch_remote(DEFAULT_UPSTREAM_REMOTE).unwrap();
    debug!("Fetched upstream remote");

    repo.create_branch_from_startpoint(
        options.branch_name,
        format!("{}/{}", DEFAULT_UPSTREAM_REMOTE, fork.default_branch).as_str(),
    )
    .unwrap();

    //had off to transformation
    //FIXME actually do some changes

    //FIXME update rusty-git to ensure errors are captured
    repo.add(vec!(".")).unwrap();
    repo.commit_all(options.commit_mesage).map_err(|e| eprintln!("{:?}", e)).unwrap();
    println!("committed");

    repo.push().unwrap();
    debug!("Pushed changes to fork");

    // open PR
    let pull = github_client
        .open_pr(options.organisation, options.repository, options.pr_title)
        .unwrap();
    debug!("Opened PR");

    let url = Url::parse(pull.url.as_str()).unwrap();

    Ok(url)
}
