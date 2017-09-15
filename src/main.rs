#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env;
use std::env::VarError;
use std::io::prelude::*;
use std::fs::File;
use clap::{App, Arg};

mod github;

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";

fn main() {
    let matches = App::new("release-party-br")
        .version("0.3.1")
        .author("Matthew Mayer <matthewkmayer@gmail.com>")
        .about("Release party automation")
        .arg(
            Arg::with_name("ORG")
                .short("o")
                .long("org")
                .value_name("github org")
                .help("Github org")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("DRYRUN")
                .short("d")
                .long("dry-run")
                .help("dry-run - don't actually create PRs"),
        )
        .get_matches();

    let org = matches
        .value_of("ORG")
        .expect("Please specify a github org");
    let org_url = format!("https://api.github.com/orgs/{}/repos", org);
    let dryrun = if matches.is_present("DRYRUN") {
        true
    } else {
        false
    };

    let token = match get_github_token() {
        Ok(gh_token) => gh_token,
        Err(e) => panic!(format!("Couldn't find {}: {}", GITHUB_TOKEN, e)),
    };
    let reqwest_client = match reqwest::Client::new() {
        Ok(new_client) => new_client,
        Err(e) => panic!("Couldn't create new reqwest client: {}", e),
    };

    let repos = get_repos_we_care_about(&token, &org_url, &reqwest_client);

    let mut pr_links: Vec<Option<String>> = repos
        .into_iter()
        .map(|repo| match get_release_pr_for(&repo, &token, &reqwest_client, dryrun) {
            Some(pr_url) => Some(pr_url),
            None => None,
        })
        .collect();
    // only keep the Some(PR_URL) items:
    pr_links.retain(|maybe_pr_link| maybe_pr_link.is_some());

    print_party_links(pr_links);
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

fn get_github_token() -> Result<String, VarError> {
    env::var(GITHUB_TOKEN)
}
