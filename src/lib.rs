use futures::Stream;
use tokio::runtime::current_thread::Runtime;
use futures::future::Future;

use hubcaps::repositories::{ForkListOptions, Repo};
use hubcaps::{Credentials, Github, Result};

pub fn create_pr(organisation: &str, repository: &str) -> Result<()> {
    let github = Github::new(
        "my-cool-user-agent/0.1.0",
        Credentials::Token("personal-access-token".into()),
      )?;

      let username = get_username(&github)?;

      let fork = existing_fork(username.as_str(), &github, organisation, repository)?;

      let fork = if let Some(existing) = fork {
          existing
      } else {
        create_fork(&github, organisation, repository)?
      };
    
      //clone fork
    
      //check if upstream remote exists, if not add
    
      //fetch upstream remote
    
      //create branch from upstream master
    
      //had off to transformation
    
      // add, commit
    
      // push
    
      // open PR

      Ok(())
}

//FIXME why does user have to be static?
fn existing_fork(user: &str, github: &Github, organisation: &str, repository: &str) -> Result<Option<Repo>> {
    let mut rt = Runtime::new()?;

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

//FIXME can we share our rt?
fn create_fork(github: &Github, organisation: &str, repository: &str) -> Result<Repo> {
    let mut rt = Runtime::new()?;
    
    rt.block_on(
        github
            .repo(organisation, repository)
            .forks()
            .create()
    )
}

fn get_username(github: &Github) -> Result<String> {
    let mut rt = Runtime::new()?;
    
    rt.block_on(
        github
            .users()
            .authenticated()
            .map(move |authed| authed.login)
    )
}