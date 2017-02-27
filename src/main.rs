#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env;
use std::env::VarError;
use std::io::prelude::*;
use std::fs::File;

mod github;

fn main() {
    let token = get_github_token().unwrap();
    let repos = github::get_repos_at("https://api.github.com/users/matthewkmayer/repos", &token).unwrap();

    let repos_to_ignore = ignored_repos();

    for repo in &repos {
        if repos_to_ignore.contains(&repo.name) {
            println!("skipping {}", repo.name);
            continue;
        }
        match github::existing_release_pr_location(repo, &token) {
            Some(url) => println!("release PR present at {}", url),
            None => match github::create_release_pull_request(repo, &token) {
                        Ok(pr_url) => println!("Made PR for {} at {}", repo.name, pr_url),
                        Err(e) => println!("Couldn't create PR for {}: {}", repo.name, e),
                    },
        }

    }
}

#[derive(Deserialize, Debug)]
struct IgnoredRepo {
    ignore: Option<Vec<String>>
}

fn ignored_repos() -> Vec<String> {
    let mut f = File::open("ignoredrepos.toml").unwrap();
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).unwrap();

    let ignored: IgnoredRepo = toml::from_str(&buffer).unwrap();
    match ignored.ignore {
        Some(repos_to_ignore) => repos_to_ignore,
        None => Vec::new(),
    }
}

fn get_github_token() -> Result<String, VarError> {
    let key = "RP_GITHUBTOKEN";
    env::var(key)
}
