use hubcaps::{Credentials, Github};

pub fn create_pr(organisation: &str, repository: &str) {
    let github = Github::new(
        "my-cool-user-agent/0.1.0",
        Credentials::Token("personal-access-token".into()),
      );
    
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
}