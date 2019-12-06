use futures::Stream;
use tokio::runtime::current_thread::Runtime;
use futures::future::Future;
use rustygit::{Repository, types::GitUrl};
use std::str::FromStr;
use tempdir::TempDir;
use url::Url;

use hubcaps::repositories::{ForkListOptions, Repo};
use hubcaps::{Credentials, Github, Result, pulls::{PullOptions, Pull}};

const DEFAULT_UPSTREAM_REMOTE: &str = "upstream";

pub fn create_pr(organisation: &str, repository: &str) -> Result<Url> {
    let mut rt = Runtime::new()?;

    let github = Github::new(
        "my-cool-user-agent/0.1.0",
        Credentials::Token("personal-access-token".into()),
      )?;

      let username = get_username(&mut rt, &github)?;

      let fork = existing_fork(&mut rt, username.as_str(), &github, organisation, repository)?;

      let fork = if let Some(existing) = fork {
          existing
      } else {
        create_fork(&mut rt, &github, organisation, repository)?
      };

      //FIXME allow users to specify path
      let tmp_dir = TempDir::new("example")?;


      //FIXME allow user to specify SSH or HTTPS
      let url = GitUrl::from_str(&fork.ssh_url).expect("github returned malformed clone URL");

      let repo = Repository::clone(url, tmp_dir.path()).unwrap();
    
      //FIXME check if upstream remote exists

      let upstream = GitUrl::from_str(format!("").as_str()).unwrap();
      repo.add_remote(DEFAULT_UPSTREAM_REMOTE, &upstream).unwrap();

      repo.fetch_remote(DEFAULT_UPSTREAM_REMOTE).unwrap();
    
      //FIXME take parameter of branch name
      repo.create_branch_from_startpoint("somebranchnamehere", format!("{}/{}", DEFAULT_UPSTREAM_REMOTE, fork.default_branch).as_str()).unwrap();
    
      //had off to transformation
    
      //FIXME need commit message
      repo.commit_all("somecommitmessage").unwrap();
    
      repo.push().unwrap();
    
      // open PR
      let pull = open_pr(&mut rt, &github, organisation, repository, "sometitlehere").unwrap();

      let url = Url::parse(pull.url.as_str()).unwrap();

      Ok(url)
}

fn open_pr(rt: &mut Runtime, github: &Github, organisation: &str, repository: &str, title: &str) -> Result<Pull> {

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

fn existing_fork(rt: &mut Runtime, user: &str, github: &Github, organisation: &str, repository: &str) -> Result<Option<Repo>> {
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

fn create_fork(rt: &mut Runtime, github: &Github, organisation: &str, repository: &str) -> Result<Repo> {
    rt.block_on(
        github
            .repo(organisation, repository)
            .forks()
            .create()
    )
}

fn get_username(rt: &mut Runtime, github: &Github) -> Result<String> {
    rt.block_on(
        github
            .users()
            .authenticated()
            .map(move |authed| authed.login)
    )
}