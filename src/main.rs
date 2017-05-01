#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate rayon;

use std::env;
use std::env::VarError;
use std::io::prelude::*;
use std::fs::File;
use rayon::prelude::*;

mod github;

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";
static GITHUB_ORG_URL: &'static str = "RP_ORGURL";
static DRYRUN: &'static str = "RP_DRYRUN";

fn main() {
    let token = match get_github_token() {
        Ok(gh_token) => gh_token,
        Err(e) => panic!(format!("Couldn't find {}: {}", GITHUB_TOKEN, e)),
    };
    let repos = get_repos_we_care_about(&token);

    let mut pr_links: Vec<Option<String>> = repos.into_par_iter()
        .map(|repo| match get_release_pr_for(&repo, &token) {
                 Some(pr_url) => Some(pr_url),
                 None => None,
             })
        .collect();
    // only keep the Some(PR_URL) items:
    pr_links.retain(|maybe_pr_link| maybe_pr_link.is_some());

    print_party_links(pr_links);
}

fn get_repos_we_care_about(token: &str) -> Vec<github::GithubRepo> {
    let github_url = match get_github_org_url() {
        Ok(url) => url,
        Err(e) => panic!(format!("Couldn't get github url to check from {}: {}", GITHUB_ORG_URL, e)),
    };
    let mut repos = match github::get_repos_at(&github_url, token) {
        Ok(repos) => repos,
        Err(e) => panic!(format!("Couldn't get repos from github: {}", e)),
    };
    let repos_to_ignore = ignored_repos();

    // remove repos we don't care about:
    repos.retain(|repo| !repos_to_ignore.contains(&repo.name));

    repos
}

fn get_release_pr_for(repo: &github::GithubRepo, token: &str) -> Option<String> {
    println!("looking for release PR for {}", repo.name);
    match github::existing_release_pr_location(repo, token) {
        Some(url) => Some(url),
        None => {
            if !github::is_release_up_to_date_with_master(&repo.url, token) {
                if dryrun() {
                    Some(format!("Dry run: {} would get a release PR.", repo.url))
                }
                else {
                    match github::create_release_pull_request(repo, token) {
                        Ok(pr_url) => Some(pr_url),
                        Err(_) => None,
                    }
                }
            } else {
                None
            }
        }
    }
}

fn print_party_links(pr_links: Vec<Option<String>>) {
    if !pr_links.is_empty() {
        println!("\nIt's a release party!  PRs to review and approve:");
        for link in pr_links {
            match link {
                Some(pr_link) => println!("{}", pr_link),
                None => println!("Party link is None: this shouldn't happen."),
            }
        }
    } else {
        println!("\nNo party today, all releases are done.");
    }
}

#[derive(Deserialize, Debug)]
struct IgnoredRepo {
    ignore: Option<Vec<String>>,
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
            println!("Couldn't parse toml from ignoredrepos.toml, not ignoring any repos. Reason: {}",
                     e);
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

fn dryrun() -> bool {
    match env::var(DRYRUN) {
        Ok(dryrun_val) => dryrun_val.parse::<bool>().unwrap_or(false),
        Err(_) => false
    }
}
