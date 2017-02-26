
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::env;
use std::env::VarError;

mod github;

fn main() {
    let token = get_github_token().unwrap();
    println!("Would use this github token: {}", token);

    // get list of repos from location foo:
    let repos = github::get_repos_at("https://api.github.com/users/matthewkmayer/repos").unwrap();

    // for each repo, attempt to create release PR:
    for repo in &repos {
        match github::create_release_pull_request(repo) {
            Ok(pr_url) => println!("Made PR for {}: {}", repo.name, pr_url),
            Err(e) => println!("Couldn't create PR for {}: {}", repo.name, e),
        }
    }
}

fn get_github_token() -> Result<String, VarError> {
    let key = "RP_GITHUBTOKEN";
    env::var(key)
}
