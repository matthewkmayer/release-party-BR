
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::env::VarError;

mod github;

fn main() {
    let token = get_github_token().unwrap();
    let repos = github::get_repos_at("https://api.github.com/users/matthewkmayer/repos", &token).unwrap();

    for repo in &repos {
        match github::existing_release_pr_location(repo, &token) {
            Some(url) => println!("release PR present at {}", url),
            None => match github::create_release_pull_request(repo, &token) {
                        Ok(pr_url) => println!("Made PR for {} at {}", repo.name, pr_url),
                        Err(e) => println!("Couldn't create PR for {}: {}", repo.name, e),
                    },
        }

    }
}

fn get_github_token() -> Result<String, VarError> {
    let key = "RP_GITHUBTOKEN";
    env::var(key)
}
