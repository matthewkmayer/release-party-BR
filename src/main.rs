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
    let repos = get_repos_we_care_about(&token);
    let mut pr_links = Vec::<String>::new();

    // prime spot for parallelizing with parallel iterators from rayon:
    for repo in &repos {
        match get_release_pr_for(repo, &token) {
            Some(pr_url) => pr_links.push(pr_url),
            None => (),
        }
    }
    print_party_links(pr_links);
}

fn get_repos_we_care_about(token: &str) -> Vec<github::GithubRepo> {
    // used "https://api.github.com/users/matthewkmayer/repos" for testing:
    let mut repos = github::get_repos_at(&get_github_org_url().unwrap(), &token).unwrap();
    let repos_to_ignore = ignored_repos();

    // remove repos we don't care about:
    repos.retain(|ref repo| !repos_to_ignore.contains(&repo.name));

    repos
}

fn get_release_pr_for(repo: &github::GithubRepo, token: &str) -> Option<String> {
     match github::existing_release_pr_location(repo, &token) {
        Some(url) => Some(url),
        None => match github::create_release_pull_request(repo, &token) {
                    Ok(pr_url) => Some(pr_url),
                    Err(_) => None,
                },
    }
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
