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
}

impl GithubClient {
    pub fn init(user_agent: &str, access_token: &str) -> Result<Self> {
        let rt = Runtime::new()?;

        let github = Github::new(user_agent, Credentials::Token(access_token.into()))?;

        Ok(GithubClient { rt, github })
    }

    pub fn open_pr(&mut self, organisation: &str, repository: &str, title: &str) -> Result<Pull> {
        //FIXME fill these in
        let options = PullOptions {
            title: String::from(title),
            head: String::from(""),
            body: None,
            base: String::from(""),
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

    pub fn get_username(&mut self) -> Result<String> {
        self.rt.block_on(
            self.github
                .users()
                .authenticated()
                .map(move |authed| authed.login),
        )
    }
}
