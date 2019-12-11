use rustygit::{types::{GitUrl, BranchName}, Repository, error::GitError as RustyGitError};
use std::str::FromStr;
use tempfile;
use url::{Url, ParseError};
use log::debug;
use github::GithubClient;
use std::path::Path;
use thiserror::Error;
use std::result::Result as stdResult;
use std::io::Error as ioError;

use hubcaps::Error as HubcapsError;

mod github;

const DEFAULT_UPSTREAM_REMOTE: &str = "upstream";

pub struct PullRequestOptions<'a> {
    pub organisation: &'a str,
    pub repository: &'a str,
    pub branch_name: &'a str,
    pub commit_mesage: &'a str,
    pub pr_title: &'a str,
}

//FIXME add messages from sources
#[derive(Debug, Error)]
pub enum PullRequestError {
    #[error("Error calling GitHub")]
    GithubError(#[from] HubcapsError),
    #[error("Git error")]
    GitError(#[from] RustyGitError),
    #[error("Unable to create temporary directory")]
    TemporaryDirectoryError(#[from] ioError),
    #[error("Unable to parse pull request URL")]
    UrlError(#[from] ParseError)
}

type Result<T> = stdResult<T, PullRequestError>; 

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
    let url = GitUrl::from_str(&fork.clone_url).expect("github returned malformed clone URL");
    debug!("Cloning repo to {:?}", tmp_dir.path());

    let repo = Repository::clone(url, tmp_dir.path())?;

    //FIXME check if upstream remote exists

    //FIXME support ssh URLs as well, what about custom githubs?
    let upstream = GitUrl::from_str(format!("https://github.com/{}/{}.git", options.organisation, options.repository).as_str())?;
    repo.add_remote(DEFAULT_UPSTREAM_REMOTE, &upstream)?;

    repo.fetch_remote(DEFAULT_UPSTREAM_REMOTE)?;
    debug!("Fetched upstream remote");

    repo.create_branch_from_startpoint(
        options.branch_name,
        format!("{}/{}", DEFAULT_UPSTREAM_REMOTE, fork.default_branch).as_str(),
    )?;

    transform(tmp_dir.path())?;

    //FIXME update rusty-git to ensure errors are captured
    repo.add(vec!("."))?;
    repo.commit_all(options.commit_mesage)?;


    //FIXME should do this validation at the start
    let upstream_branch = BranchName::from_str(options.branch_name)?;
    repo.push_to_upstream("origin", &upstream_branch)?;
    debug!("Pushed changes to fork");

    let base_branch = github_client.default_branch(options.organisation, options.repository)?;
    let pull = github_client
        .open_pr(options.organisation, options.repository, options.pr_title, base_branch.as_str(), &username, options.branch_name)?;
    debug!("Opened PR");

    let url = Url::parse(pull.url.as_str())?;

    Ok(url)
}
