use rustygit::{types::GitUrl, Repository};
use std::str::FromStr;
use tempdir::TempDir;
use url::Url;
use log::debug;

use hubcaps::Result;

mod github;

const DEFAULT_UPSTREAM_REMOTE: &str = "upstream";

pub fn create_pr(organisation: &str, repository: &str, branch_name: &str, commit_mesage: &str, pr_title: &str) -> Result<Url> {
    let mut github_client =
        github::GithubClient::init("my-cool-user-agent/0.1.0", "personal-access-token")?;

    let username = github_client.get_username()?;
    debug!("Retrieved username for github account: {}", username);

    let fork = github_client.existing_fork(username.as_str(), organisation, repository)?;

    let fork = if let Some(existing) = fork {
        existing
    } else {
        debug!("No fork exists, forking");
        github_client.create_fork(organisation, repository)?
    };

    //FIXME allow users to specify path
    let tmp_dir = TempDir::new("example")?;

    //FIXME allow user to specify SSH or HTTPS
    let url = GitUrl::from_str(&fork.ssh_url).expect("github returned malformed clone URL");

    let repo = Repository::clone(url, tmp_dir.path()).unwrap();

    //FIXME check if upstream remote exists

    let upstream = GitUrl::from_str(format!("").as_str()).unwrap();
    repo.add_remote(DEFAULT_UPSTREAM_REMOTE, &upstream).unwrap();

    repo.fetch_remote(DEFAULT_UPSTREAM_REMOTE).unwrap();

    repo.create_branch_from_startpoint(
        branch_name,
        format!("{}/{}", DEFAULT_UPSTREAM_REMOTE, fork.default_branch).as_str(),
    )
    .unwrap();

    //had off to transformation

    repo.commit_all(commit_mesage).unwrap();

    repo.push().unwrap();

    // open PR
    let pull = github_client
        .open_pr(organisation, repository, pr_title)
        .unwrap();

    let url = Url::parse(pull.url.as_str()).unwrap();

    Ok(url)
}
