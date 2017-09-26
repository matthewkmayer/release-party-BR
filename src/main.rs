#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env;
use std::io::prelude::*;
use std::fs::File;
use clap::{App, Arg};

mod github;

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";

fn main() {
    let yaml = load_yaml!("release-party.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let org_url = make_org_url(&matches);
    let token = get_github_token();
    let reqwest_client = get_reqwest_client();

    print_party_links(get_pr_links(get_repos_we_care_about(&token, &org_url, &reqwest_client), &token, &reqwest_client, is_dryrun(&matches)));
}

fn is_dryrun(matches: &clap::ArgMatches) -> bool {
    if matches.is_present("DRYRUN") {
        true
    } else {
        false
    }
}

fn make_org_url(matches: &clap::ArgMatches) -> String {
    let org = matches
        .value_of("ORG")
        .expect("Please specify a github org");

    format!("https://api.github.com/orgs/{}/repos", org)
}

fn get_pr_links(repos: Vec<github::GithubRepo>, token: &str, reqwest_client: &reqwest::Client, dryrun: bool) -> Vec<Option<String>> {
    let mut pr_links: Vec<Option<String>>  = repos.into_iter()
        .map(|repo| match get_release_pr_for(&repo, &token, &reqwest_client, dryrun) {
            Some(pr_url) => Some(pr_url),
            None => None,
        })
        .collect();
    // only keep the Some(PR_URL) items:
    pr_links.retain(|maybe_pr_link| maybe_pr_link.is_some());
    pr_links
}

fn get_reqwest_client() -> reqwest::Client {
    match reqwest::Client::new() {
        Ok(new_client) => new_client,
        Err(e) => panic!("Couldn't create new reqwest client: {}", e),
    }
}

fn get_repos_we_care_about(token: &str, github_org_url: &str, reqwest_client: &reqwest::Client) -> Vec<github::GithubRepo> {
    let mut repos = match github::get_repos_at(&github_org_url, token, reqwest_client) {
        Ok(repos) => repos,
        Err(e) => panic!(format!("Couldn't get repos from github: {}", e)),
    };
    let repos_to_ignore = ignored_repos();

    // remove repos we don't care about:
    repos.retain(|repo| !repos_to_ignore.contains(&repo.name));

    repos
}

fn get_release_pr_for(repo: &github::GithubRepo, token: &str, client: &reqwest::Client, dryrun: bool) -> Option<String> {
    println!("looking for release PR for {}", repo.name);
    match github::existing_release_pr_location(repo, token, client) {
        Some(url) => Some(url),
        None => if !github::is_release_up_to_date_with_master(&repo.url, token, client) {
            if dryrun {
                Some(format!("Dry run: {} would get a release PR.", repo.url))
            } else {
                match github::create_release_pull_request(repo, token, client) {
                    Ok(pr_url) => Some(pr_url),
                    Err(_) => None,
                }
            }
        } else {
            None
        },
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
            println!(
                "Couldn't parse toml from ignoredrepos.toml, not ignoring any repos. Reason: {}",
                e
            );
            return Vec::new();
        }
    };
    match ignored.ignore {
        Some(repos_to_ignore) => repos_to_ignore,
        None => Vec::new(),
    }
}

fn get_github_token() -> String {
    match env::var(GITHUB_TOKEN){
        Ok(gh_token) => gh_token,
        Err(e) => panic!(format!("Couldn't find {}: {}", GITHUB_TOKEN, e)),
    }
}
