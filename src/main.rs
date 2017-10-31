#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate toml;

#[cfg(test)]
#[macro_use]
extern crate hyper;

extern crate rusoto_core;
extern crate rusoto_polly;
extern crate rodio;

use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::{thread, time};
use std::default::Default;
use clap::App;
use rusoto_core::{default_tls_client, DefaultCredentialsProvider, Region};
use rusoto_polly::{Polly, PollyClient, SynthesizeSpeechInput};
use rodio::Source;

mod github;

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";

fn main() {
    let yaml = load_yaml!("release-party.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let org_url = make_org_url(&matches);
    let token = match env::var(GITHUB_TOKEN) {
        Ok(env_var) => env_var,
        Err(_) => {
            print_message_and_exit(&format!("{} environment variable should be set", GITHUB_TOKEN), -1);
            unreachable!();
        },
    };
    let reqwest_client = get_reqwest_client();

    print_party_links(get_pr_links(
        &get_repos_we_care_about(&token, &org_url, &reqwest_client),
        &token,
        &reqwest_client,
        is_dryrun(&matches),
    ));
}

fn speak_polly_speak(to_speak: &str) {
    let credentials = DefaultCredentialsProvider::new().unwrap();
    let client = PollyClient::new(default_tls_client().unwrap(), credentials, Region::UsEast1);
    let request = SynthesizeSpeechInput {
        output_format: "ogg_vorbis".to_owned(),
        text: to_speak.to_owned(),
        voice_id: "Matthew".to_owned(),
        ..Default::default()
    };
    let response = client.synthesize_speech(&request).expect("Make sounds");

    let mut buffer = File::create("foo.ogg").expect("couldn't make file");
    buffer.write(&response.audio_stream.expect("no stream found")).expect("couldn't write voice to file");
    buffer.flush().expect("couldn't flush buffer");

    let file = File::open("foo.ogg").unwrap();

    let endpoint = rodio::get_default_endpoint().expect("default endpoint?");
    let source = rodio::Decoder::new(BufReader::new(file)).expect("decoder fail");
    rodio::play_raw(&endpoint, source.convert_samples());
}

fn is_dryrun(matches: &clap::ArgMatches) -> bool {
    matches.is_present("DRYRUN")
}

fn org_is_just_org(org: &str) -> bool {
    if org.contains("https://api.github.com") {
        return false;
    }
    true
}

fn suggest_org_arg(org: &str) -> Result<String, String> {
    if org.starts_with("https://api.github.com/orgs/") && org.ends_with("/repos") {
        let suggestion = org.replace("https://api.github.com/orgs/", "").replace("/repos", "");
        return Ok(format!("{}", suggestion).to_owned());
    }
    Err("Can't make a suggestion".to_owned())
}

fn make_org_url(matches: &clap::ArgMatches) -> String {
    let org = matches
        .value_of("ORG")
        .expect("Please specify a github org");

    if !org_is_just_org(&org) {
        match suggest_org_arg(&org) {
            Ok(suggestion) => print_message_and_exit(&format!("Try this for the org value: {}", suggestion), -1),
            Err(_) => print_message_and_exit(&format!("Please make org just the organization name."), -1),
        }
    }

    format!("https://api.github.com/orgs/{}/repos", org)
}

fn get_pr_links(repos: &Vec<github::GithubRepo>, token: &str, reqwest_client: &reqwest::Client, dryrun: bool) -> Vec<Option<String>> {
    let mut pr_links: Vec<Option<String>> = repos
        .into_iter()
        .map(|repo| match get_release_pr_for(&repo, &token, reqwest_client, dryrun) {
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
    let mut repos = match github::get_repos_at(github_org_url, token, reqwest_client) {
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

// TODO: what we print and what we say should be different
fn print_party_links(pr_links: Vec<Option<String>>) {
    let mut announcement = String::new();
    if !pr_links.is_empty() {
        announcement.push_str("\nIt's a release party!  PRs to review and approve:");
        for link in pr_links {
            match link {
                Some(pr_link) => announcement.push_str(&format!("{}", pr_link)),
                None => announcement.push_str(&format!("Party link is None: this shouldn't happen.")),
            }
        }
    } else {
        announcement.push_str("\nNo party today, all releases are done.");
    }
    speak_polly_speak(&announcement);
    println!("{}", announcement);
    // TODO: wait for rodio to finish up before exiting.  This is a complete hack:
    thread::sleep(time::Duration::from_secs(10));
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

fn print_message_and_exit(message: &str, exit_code: i32) {
    println!("{}", message);
    ::std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_ignored_repos_happy_path() {
        let ignored_repositories = vec!["calagator".to_owned(), "moe".to_owned()];
        assert_eq!(ignored_repositories, ignored_repos());
    }

    #[test]
    fn handle_malformed_org() {
        assert_eq!(false, org_is_just_org("https://api.github.com/orgs/ORG-HERE/repos"));
    }

    #[test]
    fn handle_okay_org() {
        assert_eq!(true, org_is_just_org("ORG-HERE"));
    }

    #[test]
    fn suggestion_for_org_happy() {
        assert_eq!("ORG-HERE", suggest_org_arg("https://api.github.com/orgs/ORG-HERE/repos").unwrap());
    }

    #[test]
    fn suggestion_for_org_sad() {
        assert_eq!(true, suggest_org_arg("https://api.github.com/orgs/ORG-HERE/").is_err());
        assert_eq!(true, suggest_org_arg("http://api.github.com/orgs/ORG-HERE/").is_err());
        assert_eq!(true, suggest_org_arg("api.github.com/orgs/ORG-HERE/repos").is_err());
    }
}