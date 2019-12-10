use rustygit::{types::GitUrl, Repository};
use std::str::FromStr;
use tempfile;
use url::Url;
use log::debug;
use github::GithubClient;
use std::path::Path;

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

// FIXME Use traits to make params more flexible
pub fn create_enterprise_pr<F>(github_token: &str, user_agent: &str, host: &str, options: &PullRequestOptions, transform: F) -> Result<Url> 
where F: Fn(&Path) -> Result<()>
{
    let github_client =
        GithubClient::init(user_agent, github_token, Some(host))?;

    pr(github_client, options, transform)
}

pub fn create_pr<F>(github_token: &str, user_agent: &str, options: &PullRequestOptions, transform: F) -> Result<Url> 
where F: Fn(&Path) -> Result<()>
{
    let github_client =
        GithubClient::init(user_agent, github_token, None)?;

    pr(github_client, options, transform)
}

fn pr<F>(mut github_client: GithubClient, options: &PullRequestOptions, transform: F) -> Result<Url>
    where F: Fn(&Path) -> Result<()>
{
    //FIXME validate that strings are not empty

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
    //FIXME actually do some change
    transform(tmp_dir.path()).unwrap();

    //FIXME update rusty-git to ensure errors are captured
    repo.add(vec!(".")).unwrap();
    repo.commit_all(options.commit_mesage).map_err(|e| eprintln!("{:?}", e)).unwrap();
    println!("committed");//FIXME remove this line after debugging

    repo.push().unwrap();
    debug!("Pushed changes to fork");

    let base_branch = github_client.default_branch(options.organisation, options.repository)?;
    let pull = github_client
        .open_pr(options.organisation, options.repository, options.pr_title, base_branch.as_str(), &username, options.branch_name)
        .unwrap();
    debug!("Opened PR");

    let url = Url::parse(pull.url.as_str()).unwrap();

    Ok(url)
}
