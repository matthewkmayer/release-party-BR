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

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";
static GITHUB_ORG_URL: &'static str = "RP_ORGURL";

fn main() {
    let token = match get_github_token() {
        Ok(gh_token) => gh_token,
        Err(e) => panic!(format!("Couldn't find {}: {}", GITHUB_TOKEN, e)),
    };
    println!("Getting repos we care about.");
    let repos = get_repos_we_care_about(&token);
    let mut pr_links = Vec::<String>::new();

    println!("iterating over repos.");
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
    let github_url = match get_github_org_url() {
        Ok(url) => url,
        Err(e) => panic!(format!("Couldn't get github url to check from {}: {}", GITHUB_ORG_URL, e)),
    };
    let mut repos = match github::get_repos_at(&github_url, &token) {
        Ok(repos) => repos,
        Err(e) => panic!(format!("Couldn't get repos from github: {}", e)),
    };
    let repos_to_ignore = ignored_repos();

    // remove repos we don't care about:
    repos.retain(|ref repo| !repos_to_ignore.contains(&repo.name));

    repos
}

fn get_release_pr_for(repo: &github::GithubRepo, token: &str) -> Option<String> {
    println!("looking for release PR for {}", repo.name);
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
    let mut f = match File::open("ignoredrepos.toml") {
        Ok(file) => file,
        Err(e) => {
            println!("Couldn't load ignoredrepos.toml, not ignoring any repos. Reason: {}", e);
            return Vec::new();
        }
    };
    let mut buffer = String::new();
    match f.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => {
            println!("Couldn't read from ignoredrepos.toml, not ignoring any repos. Reason: {}", e);
            return Vec::new();
        }
    }

    let ignored: IgnoredRepo = match toml::from_str(&buffer) {
        Ok(ignored_repos) => ignored_repos,
        Err(e) => {
            println!("Couldn't parse toml from ignoredrepos.toml, not ignoring any repos. Reason: {}", e);
            return Vec::new();
        }
    };
    match ignored.ignore {
        Some(repos_to_ignore) => repos_to_ignore,
        None => Vec::new(),
    }
}

fn get_github_org_url() -> Result<String, VarError> {
    // used "https://api.github.com/users/matthewkmayer/repos" for testing:
    env::var(GITHUB_ORG_URL)
}

fn get_github_token() -> Result<String, VarError> {
    env::var(GITHUB_TOKEN)
}
