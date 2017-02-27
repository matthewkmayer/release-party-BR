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
    // used "https://api.github.com/users/matthewkmayer/repos" for testing:
    let repos = github::get_repos_at(&get_github_org_url().unwrap(), &token).unwrap();
    let repos_to_ignore = ignored_repos();
    let mut pr_links = Vec::<String>::new();

    for repo in &repos {
        if repos_to_ignore.contains(&repo.name) {
            println!("skipping {}", repo.name);
            continue;
        }
        match github::existing_release_pr_location(repo, &token) {
            Some(url) => pr_links.push(url),
            None => match github::create_release_pull_request(repo, &token) {
                        Ok(pr_url) => pr_links.push(pr_url),
                        Err(e) => println!("Couldn't create PR for {}: {}", repo.name, e),
                    },
        }

    }
    print_party_links(pr_links);
}

fn print_party_links(pr_links: Vec<String>) {
    if pr_links.len() > 0 {
        println!("\nIt's a release party!  PRs to review and approve:");
        for link in &pr_links {
            println!("{}", link);
        }
    }
    else {
        println!("\nNo party today, all releases are done.");
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

fn get_github_org_url() -> Result<String, VarError> {
    env::var("RP_ORGURL")
}

fn get_github_token() -> Result<String, VarError> {
    env::var("RP_GITHUBTOKEN")
}
