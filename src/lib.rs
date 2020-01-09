use github::GithubClient;
use log::debug;
use rustygit::{
    error::GitError as RustyGitError,
    types::GitUrl,
    Repository as GitRepository,
};
pub use rustygit::types::BranchName;
use std::io::Error as ioError;
use std::path::Path;
use std::result::Result as stdResult;
use std::str::FromStr;
use std::fs;
use tempfile;
use thiserror::Error;
use url::{ParseError, Url};
pub use github::GithubRepository;
use std::collections::HashSet;
use std::error::Error;

use hubcaps::Error as HubcapsError;

mod github;

const DEFAULT_UPSTREAM_REMOTE: &str = "upstream";

type TransformResult = stdResult<(), Box<dyn Error>>;

pub struct PullRequestOptions {
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
    #[error("Unable to apply transformaiton to repo: {0}")]
    TransformError(#[from] Box<dyn Error>)
}

type Result<T> = stdResult<T, PullRequestError>;

// FIXME Use traits to make params more flexible
pub fn create_enterprise_prs<F, P>(
    github_token: &str,
    user_agent: &str,
    api_endpoint: &str,
    options: &PullRequestOptions,
    transform: F,
    targets: HashSet<GithubRepository>,
    workspace: Option<P>
) -> Result<Vec<Result<Url>>>
where
    F: Fn(&Path) -> TransformResult,
    P: AsRef<Path>
{
    let github_client = GithubClient::init(user_agent, github_token, Some(api_endpoint))?;

    pr_in_workspace(github_client, options, transform, targets, workspace)
}

pub fn create_enterprise_pr<F, P>(
    github_token: &str,
    user_agent: &str,
    api_endpoint: &str,
    options: &PullRequestOptions,
    transform: F,
    target: GithubRepository,
    workspace: Option<P>
) -> Result<Url>
where
    F: Fn(&Path) -> TransformResult,
    P: AsRef<Path>
{
    let mut targets = HashSet::new();
    targets.insert(target);
    let prs = create_enterprise_prs(github_token, user_agent, api_endpoint, options, transform, targets, workspace);
    
    prs.and_then(|mut results| results.remove(0))
}

pub fn create_prs<F, P>(
    github_token: &str,
    user_agent: &str,
    options: &PullRequestOptions,
    transform: F,
    targets: HashSet<GithubRepository>,
    workspace: Option<P>
) -> Result<Vec<Result<Url>>>
where
    F: Fn(&Path) -> TransformResult,
    P: AsRef<Path>
{
    let github_client = GithubClient::init(user_agent, github_token, None)?;

    pr_in_workspace(github_client, options, transform, targets, workspace)
}

pub fn create_pr<F, P>(
    user_agent: &str,
    api_endpoint: &str,
    options: &PullRequestOptions,
    transform: F,
    target: GithubRepository,
    workspace: Option<P>
) -> Result<Url>
where
    F: Fn(&Path) -> TransformResult,
    P: AsRef<Path>
{
    let mut targets = HashSet::new();
    targets.insert(target);
    let prs = create_prs(user_agent, api_endpoint, options, transform, targets, workspace);
    
    prs.and_then(|mut results| results.remove(0))
}

fn pr_in_workspace<F, P>(github_client: GithubClient, options: &PullRequestOptions, transform: F, repositories: HashSet<GithubRepository>, workspace: Option<P>) -> Result<Vec<Result<Url>>> 
where
    F: Fn(&Path) -> TransformResult,
    P: AsRef<Path>
{
        if let Some(workspace) = workspace {
            fs::create_dir_all(workspace.as_ref())?;
            pr(github_client, options, transform, repositories, workspace.as_ref())
        } else {
            //FIXME communicate to user that dir will be deleted on dry-run
            let tmp_dir = tempfile::tempdir()?;
            pr(github_client, options, transform, repositories, tmp_dir.path())
            
        }
}

fn prepare_fork(github_client: &mut GithubClient, options: &PullRequestOptions, repository: &GithubRepository, repo_dir: &Path, username: &str) -> Result<GitRepository> {
    //FIXME validate that strings are not empty

    let fork =
        github_client.existing_fork(username, &repository)?;

    let fork = if let Some(existing) = fork {
        existing
    } else {
        debug!("No fork exists, forking");
        github_client.create_fork(&repository)?
    };

    let url = GitUrl::from_str(&fork.ssh_url).expect("github returned malformed clone URL");
    debug!("Cloning repo to {:?}", repo_dir);

    let repo = GitRepository::clone(url, repo_dir)?;

    //FIXME check if upstream remote exists

    //FIXME support https URLs as well
    let upstream = GitUrl::from_str(
        format!(
            "git@{}:{}/{}.git",
            github_client.get_host(),
            repository.organisation,
            repository.repository
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

fn submit_pr(repo: &GitRepository, github_client: &mut GithubClient, options: &PullRequestOptions, username: &str, repository: &GithubRepository) -> Result<Url> {
    //FIXME update rusty-git to ensure errors are captured
    repo.add(vec!["."])?;
    repo.commit_all(&options.commit_mesage)?;

    repo.push_to_upstream("origin", &options.branch_name)?;
    debug!("Pushed changes to fork");

    let base_branch = github_client.default_branch(&repository)?;
    let pull = github_client.open_pr(
        &repository,
        &options.pr_title,
        base_branch.as_str(),
        &username,
        &options.branch_name,
    )?;
    debug!("Opened PR");

    let url = Url::parse(pull.url.as_str())?;

    Ok(url)
}

fn pr<F>(mut github_client: GithubClient, options: &PullRequestOptions, transform: F, repositories: HashSet<GithubRepository>, workspace: &Path) -> Result<Vec<Result<Url>>>
where
    F: Fn(&Path) -> TransformResult,
{
    let username = github_client.get_username()?;
    debug!("Retrieved username for github account: {}", username);
    
    //FIXME report errors
    //FIXME right now failure is silent...
    let successful_transforms: Vec<Result<(GitRepository, &GithubRepository)>> = repositories.iter().map(|ghrepo| -> Result<(GitRepository, &GithubRepository)> {
        let repo_path = workspace.join(ghrepo.path_fragment());
        let repo = prepare_fork(&mut github_client, options, ghrepo, repo_path.as_path(), &username)?;

        transform(repo_path.as_path()).map(|_| (repo, ghrepo)).map_err(|e| e.into())
    }).collect();

    //FIXME stop here for dry-run

    successful_transforms.into_iter().map(|res| res.map(|(repo, ghrepo)| submit_pr(&repo, &mut github_client, options, &username, ghrepo))).collect()
}
