
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;

use std::env;
use std::env::VarError;
use std::io::Read;

mod github;

fn main() {
    let token = get_github_token().unwrap();
    println!("Would use this github token: {}", token);

    let mut resp = reqwest::get("https://api.github.com/users/matthewkmayer/repos").unwrap();
    let mut buffer = String::new();

    match resp.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response: {}", e),
    }

    let list_of_repos = github::repo_list_from_string(buffer).unwrap();

    for repo in &list_of_repos {
        println!("Does {} need a release? {}", repo.name, github::release_needed(repo));
    }
}

fn get_github_token() -> Result<String, VarError> {
    let key = "RP_GITHUBTOKEN";
    env::var(key)
}
