use futures::Stream;
use tokio::runtime::Runtime;

use hubcaps::repositories::{ForkListOptions, Repo};
use hubcaps::{Credentials, Github, Result};

pub fn create_pr(organisation: &str, repository: &str) -> Result<()> {
    let github = Github::new(
        "my-cool-user-agent/0.1.0",
        Credentials::Token("personal-access-token".into()),
      )?;

      let fork = existing_fork("keirlawson", &github, organisation, repository)?;

    //   let fork = fork.or_else(f: F)
    
      //check if exists, if not, fork
      // - check: calculate fork name, check if exists and is fork 
    
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
fn existing_fork(user: &'static str, github: &Github, organisation: &str, repository: &str) -> Result<Option<Repo>> {
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