use futures::future::Future;
use futures::Stream;
use tokio::runtime::current_thread::Runtime;
use rustygit::types::BranchName;
use std::path::PathBuf;

use hubcaps::repositories::{ForkListOptions, Repo};
use hubcaps::{
    pulls::{Pull, PullOptions},
    Credentials, Github, Result,
};

#[cfg(feature = "cli")]
use serde::Deserialize;

const DEFAULT_GITHUB_API_ENDPOINT: &str = "https://api.github.com";

pub struct GithubClient {
    rt: Runtime,
    github: Github,
    api_endpoint: String,
}

#[cfg_attr(feature = "cli", derive(Deserialize))]
#[derive(Hash, PartialEq, Eq)]
pub struct GithubRepository {
    pub organisation: String,
    pub repository: String
}

impl GithubRepository {
    pub fn path_fragment(&self) -> PathBuf {
        let mut path = PathBuf::new();

        path.push(&self.organisation);
        path.push(&self.repository);

        path
    }
}

impl GithubClient {
    pub fn init(user_agent: &str, access_token: &str, api_endpoint: Option<&str>) -> Result<Self> {
        let rt = Runtime::new()?;

        let credential = Credentials::Token(access_token.into());
        let gh_api_endpoint: String;
        let github = if let Some(api_endpoint) = api_endpoint {
            gh_api_endpoint = api_endpoint.to_string();
            Github::host(api_endpoint, user_agent, credential)
        } else {
            gh_api_endpoint = String::from(DEFAULT_GITHUB_API_ENDPOINT);
            Github::new(user_agent, credential)
        }?;

        Ok(GithubClient {
            rt,
            github,
            api_endpoint: gh_api_endpoint,
        })
    }

    pub fn open_pr(
        &mut self,
        repository: &GithubRepository,
        title: &str,
        base_branch: &str,
        username: &str,
        head_branch: &BranchName,
    ) -> Result<Pull> {
        let options = PullOptions {
            title: String::from(title),
            head: format!("{}:{}", username, head_branch),
            body: None,
            base: String::from(base_branch),
        };

        self.rt.block_on(
            self.github
                .repo(&repository.organisation, &repository.repository)
                .pulls()
                .create(&options),
        )
    }

    pub fn existing_fork(
        &mut self,
        user: &str,
        repository: &GithubRepository,
    ) -> Result<Option<Repo>> {
        let options = ForkListOptions::builder().build();
        let mut forks = self.rt.block_on(
            self.github
                .repo(&repository.organisation, &repository.repository)
                .forks()
                .iter(&options)
                .filter(move |repo| repo.owner.login == user)
                .collect(),
        )?;

        if !forks.is_empty() {
            Ok(Some(forks.remove(0)))
        } else {
            Ok(None)
        }
    }

    pub fn create_fork(&mut self, repository: &GithubRepository) -> Result<Repo> {
        self.rt
            .block_on(self.github.repo(&repository.organisation, &repository.repository).forks().create())
    }

    pub fn default_branch(&mut self, repository: &GithubRepository) -> Result<String> {
        self.rt
            .block_on(self.github.repo(&repository.organisation, &repository.repository).get())
            .map(|r| r.default_branch)
    }

    pub fn get_username(&mut self) -> Result<String> {
        self.rt.block_on(
            self.github
                .users()
                .authenticated()
                .map(move |authed| authed.login),
        )
    }

    pub fn get_host(&self) -> String {
        let host = self
            .api_endpoint
            .replace("http://", "")
            .replace("https://", "")
            .replace("api.", "");

        host.split_at(match host.find("/") {
            Some(position) => position,
            None => host.len(),
        })
        .0
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_host_works_with_gh_dot_com() {
        let client = get_client_with_endpoint(String::from("https://api.github.com"));
        assert_eq!("github.com", client.get_host());
    }

    #[test]
    fn test_get_host_works_with_ghe() {
        let client =
            get_client_with_endpoint(String::from("https://github.awesomecompany.com/api/v3"));
        assert_eq!("github.awesomecompany.com", client.get_host());
    }

    #[test]
    fn test_get_host_works_with_weird_urls() {
        let client = get_client_with_endpoint(String::from(
            "https://api.github.anothercompany.com.br/api/v2/gh/really/long/url?name=A",
        ));
        assert_eq!("github.anothercompany.com.br", client.get_host());
    }

    fn get_client_with_endpoint(endpoint: String) -> GithubClient {
        let rt = Runtime::new().unwrap();

        let credential = Credentials::Token(String::from("TOKEN TOKEN TOKEN"));
        let github = Github::new("Testing", credential).unwrap();

        GithubClient {
            rt,
            github,
            api_endpoint: endpoint,
        }
    }
}
