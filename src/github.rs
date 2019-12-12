use futures::future::Future;
use futures::Stream;
use tokio::runtime::current_thread::Runtime;

use hubcaps::repositories::{ForkListOptions, Repo};
use hubcaps::{
    pulls::{Pull, PullOptions},
    Credentials, Github, Result,
};
pub struct GithubClient {
    rt: Runtime,
    github: Github,
    api_endpoint: String,
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
            gh_api_endpoint = String::from("https://api.github.com");
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
        organisation: &str,
        repository: &str,
        title: &str,
        base_branch: &str,
        username: &str,
        head_branch: &str,
    ) -> Result<Pull> {
        let options = PullOptions {
            title: String::from(title),
            head: format!("{}:{}", username, head_branch),
            body: None,
            base: String::from(base_branch),
        };

        self.rt.block_on(
            self.github
                .repo(organisation, repository)
                .pulls()
                .create(&options),
        )
    }

    pub fn existing_fork(
        &mut self,
        user: &str,
        organisation: &str,
        repository: &str,
    ) -> Result<Option<Repo>> {
        let options = ForkListOptions::builder().build();
        let mut forks = self.rt.block_on(
            self.github
                .repo(organisation, repository)
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

    pub fn create_fork(&mut self, organisation: &str, repository: &str) -> Result<Repo> {
        self.rt
            .block_on(self.github.repo(organisation, repository).forks().create())
    }

    pub fn default_branch(&mut self, organisation: &str, repository: &str) -> Result<String> {
        self.rt
            .block_on(self.github.repo(organisation, repository).get())
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
