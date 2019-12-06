use tokio::runtime::current_thread::Runtime;
use futures::future::Future;
use futures::Stream;


use hubcaps::repositories::{ForkListOptions, Repo};
use hubcaps::{Github, Result, pulls::{PullOptions, Pull}};

pub fn open_pr(rt: &mut Runtime, github: &Github, organisation: &str, repository: &str, title: &str) -> Result<Pull> {

    //FIXME fill these in
    let options = PullOptions {
        title: String::from(title),
        head: String::from(""),
        body: None,
        base: String::from("")
    };

    rt.block_on(
        github
            .repo(organisation, repository)
            .pulls()
            .create(&options)
    )
}

pub fn existing_fork(rt: &mut Runtime, user: &str, github: &Github, organisation: &str, repository: &str) -> Result<Option<Repo>> {
    let options = ForkListOptions::builder().build();
    let mut forks = rt.block_on(
        github
            .repo(organisation, repository)
            .forks()
            .iter(&options)
            .filter(move |repo| repo.owner.login == user)
            .collect()
    )?;

    if !forks.is_empty() {
        Ok(Some(forks.remove(0)))
    } else {
        Ok(None)
    }
}

pub fn create_fork(rt: &mut Runtime, github: &Github, organisation: &str, repository: &str) -> Result<Repo> {
    rt.block_on(
        github
            .repo(organisation, repository)
            .forks()
            .create()
    )
}

pub fn get_username(rt: &mut Runtime, github: &Github) -> Result<String> {
    rt.block_on(
        github
            .users()
            .authenticated()
            .map(move |authed| authed.login)
    )
}