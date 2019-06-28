extern crate reqwest;
extern crate serde_json;

use self::reqwest::header::LINK;
use self::reqwest::{Error, Response, Url};
use reqwest::hyper_011::{header::Link, header::RelationType, Headers};

use std::collections::HashMap;
use std::io::Read;
use std::{thread, time};

#[derive(Deserialize, Debug)]
pub struct GithubRepo {
    id: i32,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct CompareCommitsResponse {
    pub status: String,
    pub behind_by: i32,
}

#[derive(Deserialize, Debug)]
pub struct GithubPullRequest {
    id: i32,
    pub url: String,
    pub html_url: String,
    pub head: Commit,
    pub base: Commit,
}

#[derive(Deserialize, Debug)]
pub struct Commit {
    pub sha: String,
}

#[derive(Deserialize, Debug)]
pub struct ActualCommitInPR {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct CommitInPR {
    pub sha: String,
    #[serde(rename = "commit")]
    pub actual_commit: ActualCommitInPR,
}

pub fn is_release_up_to_date_with_master(repo_url: &str, client: &reqwest::Client) -> bool {
    let repo_pr_url = format!("{}/{}/{}...{}", repo_url, "compare", "master", "release");
    let url = match Url::parse(&repo_pr_url) {
        Ok(new_url) => new_url,
        Err(e) => {
            println!("Couldn't create url for compare page: {}", e);
            return true;
        }
    };
    let mut res = match client.get(url.clone()).send() {
        Ok(response) => response,
        Err(e) => {
            println!("Error in request to github for compare page: {}", e);
            return true;
        }
    };
    delay_if_running_out_of_requests(res.headers());

    let mut buffer = String::new();
    match res.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error checking commit diff for {}: {}", repo_url, e),
    }

    let commits_diff: CompareCommitsResponse = match serde_json::from_str(&buffer) {
        Ok(compare_response) => compare_response,
        Err(_) => return true,
    };

    if commits_diff.behind_by > 0 {
        return false;
    }

    true
}

fn delay_if_running_out_of_requests(response_headers: &reqwest::header::HeaderMap) {
    if close_to_running_out_of_requests(response_headers) {
        println!("Running low on requests, throttling back...");
        thread::sleep(time::Duration::from_millis(2000));
    }
}

// This is a bit simplistic at the moment by trying to prevent bottoming out completely.
// An improvement could be to use the fraction of requests left over the overall limit.
// EG: 55 requests left out of a limit of 60 is fine.
// 10 requests left out of 60 is time to throttle back.
// TODO: dump the `expects` and handle things more gracefully.
fn close_to_running_out_of_requests(response_headers: &reqwest::header::HeaderMap) -> bool {
    let requests_to_treat_as_running_out = 10;
    let remaining_requests = match response_headers.get("X-RateLimit-Remaining") {
        Some(remaining_req_from_github) => {
            let req_left = remaining_req_from_github
                .to_str()
                .expect("Rate limit should be able to be a string")
                .replace('"', ""); // the formatter puts quotes around the number.  EG: "55"
            let int_req_left = req_left
                .parse::<i32>()
                .expect("Expected number in X-RateLimit-Remaining field");
            if int_req_left == 0 {
                panic!("We're out of requests, sad! Exiting poorly.");
            }
            int_req_left
        }
        // If it's not specified, we'll say we have enough to keep going:
        None => requests_to_treat_as_running_out + 1,
    };
    remaining_requests < requests_to_treat_as_running_out
}

fn response_has_a_next_link(response_headers: &reqwest::header::HeaderMap) -> bool {
    if response_headers.get(LINK).is_none() {
        return false;
    }

    let headers = Headers::from(response_headers.clone());
    if let Some(link) = headers.get::<Link>() {
        return link.values().iter().any(|ref l| {
            if let Some(r) = l.rel() {
                if r.iter().any(|foo| foo == &RelationType::Next) {
                    return true;
                }
            }
            false
        });
    }
    false
}

// Expects caller to check to ensure the `next` link is present
fn response_next_link(response_headers: &reqwest::header::HeaderMap) -> Result<Url, String> {
    let headers = Headers::from(response_headers.clone());
    if let Some(link) = headers.get::<Link>() {
        for l in link.values() {
            if let Some(r) = l.rel() {
                // r will be a collection of relations
                for rel in r {
                    if rel == &RelationType::Next {
                        let uri = Url::parse(l.link()).expect("Should have a nextlink");
                        return Ok(uri);
                    }
                }
            }
        }
    }
    Err("Couldn't find a next link: does it exist?".to_string())
}

pub fn get_repos_at(repos_url: &str, client: &reqwest::Client) -> Result<Vec<GithubRepo>, String> {
    // We need to pass in the URL from the link headers to github API docs.
    // We'll construct it this first time.
    let url = match Url::parse_with_params(repos_url, &[("per_page", "50")]) {
        Ok(new_url) => new_url,
        Err(e) => return Err(format!("Couldn't parse uri {:?} : {:?}", repos_url, e)),
    };
    let mut response = get_repos_at_url(url, client).expect("request failed");

    if !response.status().is_success() {
        let mut buffer = String::new();
        match response.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(e) => println!(
                "error reading response from github when getting repo list: {}",
                e
            ),
        }
        panic!("Error: Github responded with {:?}", buffer);
    }
    delay_if_running_out_of_requests(response.headers());
    let mut buffer = String::new();
    match response.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!(
            "error reading response from github when getting repo list: {}",
            e
        ),
    }
    let mut repos = repo_list_from_string(&buffer).expect("expected repos");

    if response_has_a_next_link(response.headers()) {
        loop {
            let paging_url = response_next_link(response.headers()).expect("a thing");
            response = get_repos_at_url(paging_url, client).expect("request failed");
            delay_if_running_out_of_requests(response.headers());
            buffer = String::new();
            match response.read_to_string(&mut buffer) {
                Ok(_) => (),
                Err(e) => println!(
                    "error reading response from github when getting repo list: {}",
                    e
                ),
            }
            repos.append(&mut repo_list_from_string(&buffer).expect("expected repos"));
            if !response_has_a_next_link(response.headers()) {
                break;
            }
        }
    }
    println!("Number of repos to check: {:?}", repos.len());
    Ok(repos)
}

fn get_repos_at_url(url: reqwest::Url, client: &reqwest::Client) -> Result<Response, Error> {
    client.get(url).send()
}

fn repo_list_from_string(json_str: &str) -> Result<Vec<GithubRepo>, String> {
    // This looks a bit weird due to supplying type hints to deserialize:
    let _: Vec<GithubRepo> = match serde_json::from_str(json_str) {
        Ok(v) => return Ok(v),
        Err(e) => return Err(format!("Couldn't deserialize repos from github: {}", e)),
    };
}

pub fn existing_release_pr_location(repo: &GithubRepo, client: &reqwest::Client) -> Option<String> {
    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let url = match Url::parse_with_params(&repo_pr_url, &[("head", "master"), ("base", "release")])
    {
        Ok(new_url) => new_url,
        Err(e) => {
            println!("Couldn't create url for existing pr location: {}", e);
            return None;
        }
    };
    let mut res = match client.get(url).send() {
        Ok(response) => response,
        Err(e) => {
            println!("Error in request to github for existing PR location: {}", e);
            return None;
        }
    };
    delay_if_running_out_of_requests(res.headers());
    let mut buffer = String::new();
    match res.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error finding existing pr for {}: {}", repo.name, e),
    }

    let pull_reqs: Vec<GithubPullRequest> = match serde_json::from_str(&buffer) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };

    if !pull_reqs.is_empty() {
        return Some(pull_reqs[0].html_url.clone());
    }

    None
}

pub fn get_commits_from_pr(
    repo: &GithubRepo,
    pr_number: &str,
    client: &reqwest::Client,
    rp_version: &str,
) -> String {
    let pr_commits_url = format!("{}/pulls/{}/commits", repo.url, pr_number);

    let mut res = match client.get(&pr_commits_url).send() {
        Ok(response) => response,
        Err(e) => {
            panic!("Error in request to github for compare page: {}", e);
        }
    };
    delay_if_running_out_of_requests(res.headers());

    let mut buffer = String::new();
    match res.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error checking commit diff for {}: {}", pr_commits_url, e),
    }

    let prs: Vec<CommitInPR> = match serde_json::from_str(&buffer) {
        Ok(v) => v,
        Err(e) => panic!(format!("Couldn't deserialize repos from github: {}. Payload: {:#?}", e, buffer)),
    };

    let mut new_body = "automated release partay!\n\nPRs in this release:".to_string();

    for c in prs.into_iter() {
        if c.actual_commit.message.contains("Merge pull request #") {
            // remove the bits we don't need: go from "Merge pull request #1890 from..." to "#1890"
            let pr_number = c.actual_commit.message.split(' ').collect::<Vec<&str>>()[3];
            new_body.push_str(&format!("\n* {}", pr_number));
        }
    }

    new_body.push_str(&format!("\n\n---\nMade by `{}`.", rp_version));

    new_body
}

pub fn set_pr_body(repo: &GithubRepo, pr_number: &str, body: &str, client: &reqwest::Client) {
    let mut pr_body = HashMap::new();
    pr_body.insert("body", body);

    let repo_pr_url = format!("{}/pulls/{}", repo.url, pr_number);
    let res = match client.patch(&repo_pr_url).json(&pr_body).send() {
        Ok(response) => response,
        Err(e) => panic!(format!("Error in request to github creating new PR: {}", e)),
    };
    delay_if_running_out_of_requests(res.headers());
}

pub fn update_pr_body(
    repo: &GithubRepo,
    pr_number: &str,
    client: &reqwest::Client,
    rp_version: &str,
) {
    let new_body = get_commits_from_pr(repo, pr_number, client, rp_version);
    set_pr_body(repo, pr_number, &new_body, client);
}

// Try to create the release PR and return the URL of it:
pub fn create_release_pull_request(
    repo: &GithubRepo,
    client: &reqwest::Client,
) -> Result<String, String> {
    let mut pr_body = HashMap::new();
    pr_body.insert("title", "automated release partay");
    pr_body.insert("head", "master");
    pr_body.insert("base", "release");

    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let mut res = match client.post(&repo_pr_url).json(&pr_body).send() {
        Ok(response) => response,
        Err(e) => return Err(format!("Error in request to github creating new PR: {}", e)),
    };

    delay_if_running_out_of_requests(res.headers());
    if res.status().is_success() {
        let mut buffer = String::new();
        match res.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(e) => println!(
                "error reading response after creating new release PR for {}: {}",
                repo.name, e
            ),
        }
        let pull_req: GithubPullRequest = match serde_json::from_str(&buffer) {
            Ok(v) => v,
            Err(e) => {
                return Err(format!(
                    "Couldn't deserialize create pull req response for {}: {}",
                    repo.name, e
                ));
            }
        };
        return Ok(pull_req.html_url);
    }
    // 422 unprocessable means it's there already
    // 422 unprocessable also means the branch is up to date

    Err("Release branch already up to date?".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;

    #[test]
    fn plenty_of_requests_left() {
        let mut plenty_left_headers = HeaderMap::new();
        plenty_left_headers.insert("X-RateLimit-Remaining", "10000".parse().unwrap());

        assert_eq!(
            false,
            close_to_running_out_of_requests(&plenty_left_headers)
        );
    }

    #[test]
    fn almost_no_requests_left() {
        let mut nothing_left_headers = HeaderMap::new();
        nothing_left_headers.insert("X-RateLimit-Remaining", "1".parse().unwrap());
        assert_eq!(
            true,
            close_to_running_out_of_requests(&nothing_left_headers)
        );
    }

    #[test]
    fn no_next_link() {
        assert_eq!(false, response_has_a_next_link(&HeaderMap::new()));
    }

    #[test]
    fn has_next_link() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            "<http://example.com/TheBook/chapter2>; rel=\"next\"; title=\"next page\""
                .parse()
                .unwrap(),
        );

        assert_eq!(true, response_has_a_next_link(&headers));
    }

    #[test]
    fn finds_next_link() {
        let mut headers = HeaderMap::new();
        headers.insert(
            LINK,
            "<http://example.com/TheBook/chapter3>; rel=\"next\"; title=\"next page\""
                .parse()
                .unwrap(),
        );

        let expected_uri = Url::parse("http://example.com/TheBook/chapter3").unwrap();
        assert_eq!(expected_uri, response_next_link(&headers).unwrap());
    }
}
