#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

#[macro_use]
extern crate clap;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate indicatif;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
extern crate hyper;

use clap::App;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use indicatif::ProgressBar;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

mod github;

static GITHUB_TOKEN: &'static str = "RP_GITHUBTOKEN";
static USERAGENT: &'static str = "release-party-br";

lazy_static! {
    static ref RP_VERSION: String = {
        let yaml = load_yaml!("release-party.yml");
        let app = App::from_yaml(yaml);
        version_string(&app)
    };
}

fn main() {
    let yaml = load_yaml!("release-party.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let org_url = make_org_url(&matches);
    let token = match env::var(GITHUB_TOKEN) {
        Ok(env_var) => env_var,
        Err(_) => {
            print_message_and_exit(
                &format!("{} environment variable should be set", GITHUB_TOKEN),
                -1,
            );
            unreachable!();
        }
    };
    let reqwest_client = get_reqwest_client(&token);

    let links = get_pr_links(
        &get_repos_we_care_about(&org_url, &reqwest_client),
        &reqwest_client,
        is_dryrun(&matches),
    );

    print_party_links(links);
}

fn version_string(app: &App) -> String {
    let mut version: Vec<u8> = Vec::new();
    app.write_version(&mut version)
        .expect("Should write to version vec.");
    String::from_utf8(version).expect("Version text should be utf8 text")
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
        let suggestion = org
            .replace("https://api.github.com/orgs/", "")
            .replace("/repos", "");
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
            Ok(suggestion) => {
                print_message_and_exit(&format!("Try this for the org value: {}", suggestion), -1)
            }
            Err(_) => {
                print_message_and_exit(&format!("Please make org just the organization name."), -1)
            }
        }
    }

    format!("https://api.github.com/orgs/{}/repos", org)
}

fn get_pr_links(
    repos: &Vec<github::GithubRepo>,
    reqwest_client: &reqwest::Client,
    dryrun: bool,
) -> Vec<Option<String>> {
    let bar = ProgressBar::new(repos.len() as u64);
    let mut pr_links: Vec<Option<String>> = repos
        .into_iter()
        .map(|repo| {
            bar.inc(1);
            let i = match get_release_pr_for(&repo, reqwest_client, dryrun) {
                Some(pr_url) => Some(pr_url),
                None => None,
            };
            // update the PR body
            // pr_url will look like https://github.com/matthewkmayer/release-party-BR/pull/39
            // split by '/' and grab last chunk.
            match i {
                Some(ref pr_url) => {
                    let pr_split = pr_url.split('/').collect::<Vec<&str>>();
                    let pr_num = pr_split.last().expect("PR link malformed?");
                    github::update_pr_body(repo, pr_num, reqwest_client, &RP_VERSION);
                }
                None => (),
            }
            i
        })
        .collect();
    bar.finish();
    // only keep the Some(PR_URL) items:
    pr_links.retain(|maybe_pr_link| maybe_pr_link.is_some());
    pr_links
}

fn get_reqwest_client(token: &str) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        USER_AGENT,
        USERAGENT.parse().expect("useragent should be a string"),
    );
    headers.insert(
        AUTHORIZATION,
        format!("token {}", token)
            .parse()
            .expect("token should be a string"),
    );
    match reqwest::Client::builder().default_headers(headers).build() {
        Ok(new_client) => new_client,
        Err(e) => panic!("Couldn't create new reqwest client: {}", e),
    }
}

fn get_repos_we_care_about(
    github_org_url: &str,
    reqwest_client: &reqwest::Client,
) -> Vec<github::GithubRepo> {
    let mut repos = match github::get_repos_at(github_org_url, reqwest_client) {
        Ok(repos) => repos,
        Err(e) => panic!(format!("Couldn't get repos from github: {}", e)),
    };
    let repos_to_ignore = ignored_repos();

    // remove repos we don't care about:
    repos.retain(|repo| !repos_to_ignore.contains(&repo.name));

    repos
}

fn get_release_pr_for(
    repo: &github::GithubRepo,
    client: &reqwest::Client,
    dryrun: bool,
) -> Option<String> {
    match github::existing_release_pr_location(repo, client) {
        Some(url) => Some(url),
        None => {
            if !github::is_release_up_to_date_with_master(&repo.url, client) {
                if dryrun {
                    Some(format!("Dry run: {} would get a release PR.", repo.url))
                } else {
                    match github::create_release_pull_request(repo, client) {
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
    let hfi = match dirs::home_dir() {
        Some(path) => {
            if Path::new(&path).join(".ignoredrepos.toml").exists() {
                Some(Path::new(&path).join(".ignoredrepos.toml"))
            } else {
                None
            }
        },
        None => None,
    };

    let lfi = match Path::new("ignoredrepos.toml").exists() {
        true  => Some(Path::new("ignoredrepos.toml").to_path_buf()),
        false => None,
    };

    let fi = match (lfi, hfi) {
        (Some(a), _) => a,
        (None, Some(b)) => b,
        (_, _) => {println!("The ignoredrepos.toml file not found"); return Vec::new()},
    };

    let mut f = match File::open(&fi) {
        Ok(file) => file,
        Err(e) => {
            println!(
                "Couldn't load ignoredrepos.toml, not ignoring any repos. Reason: {}",
                e
            );
            return Vec::new();
        }
    };

    println!(
        "Found ignoredrepos.toml file at {:#?}",
        fi
    );

    let mut buffer = String::new();
    match f.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => {
            println!(
                "Couldn't read from ignoredrepos.toml, not ignoring any repos. Reason: {}",
                e
            );
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
        assert_eq!(
            false,
            org_is_just_org("https://api.github.com/orgs/ORG-HERE/repos")
        );
    }

    #[test]
    fn handle_okay_org() {
        assert_eq!(true, org_is_just_org("ORG-HERE"));
    }

    #[test]
    fn suggestion_for_org_happy() {
        assert_eq!(
            "ORG-HERE",
            suggest_org_arg("https://api.github.com/orgs/ORG-HERE/repos").unwrap()
        );
    }

    #[test]
    fn suggestion_for_org_sad() {
        assert_eq!(
            true,
            suggest_org_arg("https://api.github.com/orgs/ORG-HERE/").is_err()
        );
        assert_eq!(
            true,
            suggest_org_arg("http://api.github.com/orgs/ORG-HERE/").is_err()
        );
        assert_eq!(
            true,
            suggest_org_arg("api.github.com/orgs/ORG-HERE/repos").is_err()
        );
    }
}
