use github::GithubClient;
use log::debug;
use rustygit::{
    error::GitError as RustyGitError,
    types::GitUrl,
    Repository,
};
pub use rustygit::types::BranchName;
use std::io::Error as ioError;
use std::path::Path;
use std::result::Result as stdResult;
use std::str::FromStr;
use tempfile;
use thiserror::Error;
use url::{ParseError, Url};

use hubcaps::Error as HubcapsError;

mod github;

const DEFAULT_UPSTREAM_REMOTE: &str = "upstream";

pub struct PullRequestOptions {
    pub organisation: String,
    pub repository: String,
    pub branch_name: BranchName,
    pub commit_mesage: String,
    pub pr_title: String,
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
    UrlError(#[from] ParseError),
}

type Result<T> = stdResult<T, PullRequestError>;

// FIXME Use traits to make params more flexible
pub fn create_enterprise_pr<F>(
    github_token: &str,
    user_agent: &str,
    api_endpoint: &str,
    options: &PullRequestOptions,
    transform: F,
) -> Result<Url>
where
    F: Fn(&Path) -> Result<()>,
{
    let github_client = GithubClient::init(user_agent, github_token, Some(api_endpoint))?;

    pr(github_client, options, transform)
}

pub fn create_pr<F>(
    github_token: &str,
    user_agent: &str,
    options: &PullRequestOptions,
    transform: F,
) -> Result<Url>
where
    F: Fn(&Path) -> Result<()>,
{
    let github_client = GithubClient::init(user_agent, github_token, None)?;

    pr(github_client, options, transform)
}

fn prepare_fork(github_client: &mut GithubClient, options: &PullRequestOptions, repo_dir: &Path, username: &str) -> Result<Repository> {
    //FIXME validate that strings are not empty

    let fork =
        github_client.existing_fork(username, &options.organisation, &options.repository)?;

    let fork = if let Some(existing) = fork {
        existing
    } else {
        debug!("No fork exists, forking");
        github_client.create_fork(&options.organisation, &options.repository)?
    };

    let url = GitUrl::from_str(&fork.ssh_url).expect("github returned malformed clone URL");
    debug!("Cloning repo to {:?}", repo_dir);

    let repo = Repository::clone(url, repo_dir)?;

    //FIXME check if upstream remote exists

    //FIXME support https URLs as well
    let upstream = GitUrl::from_str(
        format!(
            "git@{}:{}/{}.git",
            github_client.get_host(),
            options.organisation,
            options.repository
        )
        .as_str(),
    )?;
    repo.add_remote(DEFAULT_UPSTREAM_REMOTE, &upstream)?;

    repo.fetch_remote(DEFAULT_UPSTREAM_REMOTE)?;
    debug!("Fetched upstream remote");

    repo.create_branch_from_startpoint(
        &options.branch_name,
        format!("{}/{}", DEFAULT_UPSTREAM_REMOTE, fork.default_branch).as_str(),
    )?;

    Ok(repo)
}

fn submit_pr(repo: &Repository, github_client: &mut GithubClient, options: &PullRequestOptions, username: &str) -> Result<Url> {
    //FIXME update rusty-git to ensure errors are captured
    repo.add(vec!["."])?;
    repo.commit_all(&options.commit_mesage)?;

    repo.push_to_upstream("origin", &options.branch_name)?;
    debug!("Pushed changes to fork");

    let base_branch = github_client.default_branch(&options.organisation, &options.repository)?;
    let pull = github_client.open_pr(
        &options.organisation,
        &options.repository,
        &options.pr_title,
        base_branch.as_str(),
        &username,
        &options.branch_name,
    )?;
    debug!("Opened PR");

    let url = Url::parse(pull.url.as_str())?;

    Ok(url)
}

fn pr<F>(mut github_client: GithubClient, options: &PullRequestOptions, transform: F) -> Result<Url>
where
    F: Fn(&Path) -> Result<()>,
{
    let username = github_client.get_username()?;
    debug!("Retrieved username for github account: {}", username);

    //FIXME allow users to specify path
    let tmp_dir = tempfile::tempdir()?;
    let repo_dir = tmp_dir.path();
    
    let repo = prepare_fork(&mut github_client, options, repo_dir, &username)?;

    transform(repo_dir)?;

    submit_pr(&repo, &mut github_client, options, &username)
}
