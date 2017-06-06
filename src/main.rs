#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate rayon;
extern crate reqwest;
extern crate clap;

use std::env;
use std::env::VarError;
use std::io::prelude::*;
use std::fs::File;
use rayon::prelude::*;
use clap::{Arg, App};

mod github;

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";

fn main() {
    let matches = App::new("release-party-br")
        .version("0.2.0")
        .author("Matthew Mayer <matthewkmayer@gmail.com>")
        .about("Release party automation")
        .arg(Arg::with_name("ORG_URL")
            .short("o")
            .long("org_url")
            .value_name("github url")
            .help("Github org url to use.")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("DRYRUN")
            .short("d")
            .long("dry-run")
            .help("dry-run - don't actually create PRs"))
        .arg(Arg::with_name("YOLO")
            .long("yolo")
            .help("yolo - merge PRs without reviewing"))
        .get_matches();

    let org_url = matches.value_of("ORG_URL").expect("Please specify a github org");
    let dryrun = if matches.is_present("DRYRUN") { true } else { false };
    let yolo = if matches.is_present("YOLO") { true } else { false };
    if yolo { println!("LEEROY JENKINS"); }

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
        .into_par_iter()
        .map(|repo| match get_release_pr_for(&repo, &token, &reqwest_client, dryrun) {
                 Some(pr_url) => Some(pr_url),
                 None => None,
             })
        .collect();
    // only keep the Some(PR_URL) items:
    pr_links.retain(|maybe_pr_link| maybe_pr_link.is_some());

    if yolo {
        // approve and merge existing PRs
        // TODO: how to handle ones we created thus can't approve?
        // https://developer.github.com/v3/pulls/reviews/#create-a-pull-request-review
        // https://developer.github.com/v3/pulls/reviews/#submit-a-pull-request-review
        // https://developer.github.com/v3/pulls/#merge-a-pull-request-merge-button
        for repo in repos.iter() {
            approve_and_merge(&token, repo, &reqwest_client);
        }
        println!("should have tried to approve and merge.");
    }

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

fn approve_and_merge(token: &str, repo: &github::GithubRepo, client: &reqwest::Client) {
    let review_link = match github::create_pr_review(token, repo, client) {
        Ok(review_link) => Some(review_link),
        Err(e) => {
            println!("error creating pr review: {}", e);
            None
        }
    };
    println!("Review link is {:?}", review_link);
    match review_link {
        Some(review_link) => {
            match github::submit_pr_review(token, &review_link, client) {
                Ok(_) => (),
                Err(e) => println!("error submitting pr review: {}", e),
            }
            match github::merge_pr(token, &review_link, client) {
                Ok(_) => (),
                Err(e) => println!("error merging pr: {}", e),
            }
        },
        None => ()
    }
}

fn get_release_pr_for(repo: &github::GithubRepo, token: &str, client: &reqwest::Client, dryrun: bool) -> Option<String> {
    println!("looking for release PR for {}", repo.name);
    match github::existing_release_pr_location(repo, token, client) {
        Some(url) => Some(url),
        None => {
            if !github::is_release_up_to_date_with_master(&repo.url, token, client) {
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

fn get_github_token() -> Result<String, VarError> {
    env::var(GITHUB_TOKEN)
}
