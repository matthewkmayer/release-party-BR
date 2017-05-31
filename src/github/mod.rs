extern crate reqwest;
extern crate serde_json;

use self::reqwest::header::{Authorization, UserAgent};
use self::reqwest::Url;

use std::io::Read;
use std::collections::HashMap;

static USERAGENT: &'static str = "release-party-br";

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
    pub label: String,
}

pub fn is_release_up_to_date_with_master(repo_url: &str, token: &str) -> bool {
    let repo_pr_url = format!("{}/{}/{}...{}", repo_url, "compare", "master", "release");
    let url = match Url::parse(&repo_pr_url) {
        Ok(new_url) => new_url,
        Err(e) => {
            println!("Couldn't create url for compare page: {}", e);
            return true;
        }
    };
    let client = match reqwest::Client::new() {
        Ok(new_client) => new_client,
        Err(e) => {
            println!("Couldn't create new reqwest client: {}", e);
            return true;
        }
    };
    let mut res = match client
              .get(url.clone())
              .header(UserAgent(USERAGENT.to_string()))
              .header(Authorization(format!("token {}", token)))
              .send() {
        Ok(response) => response,
        Err(e) => {
            println!("Error in request to github for compare page: {}", e);
            return true;
        }
    };

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

pub fn get_repos_at(repos_url: &str, token: &str, client: &reqwest::Client) -> Result<Vec<GithubRepo>, String> {
    let url = match Url::parse_with_params(repos_url, &[("per_page", "100")]) {
        Ok(new_url) => new_url,
        Err(e) => return Err(format!("Couldn't create url for getting repo list: {}", e)),
    };
    println!("Getting repos at {}", url);
    let mut resp = match client
              .get(url)
              .header(UserAgent(USERAGENT.to_string()))
              .header(Authorization(format!("token {}", token)))
              .send() {
        Ok(response) => response,
        Err(e) => return Err(format!("Error in request to github to get repos: {}", e)),
    };

    let mut buffer = String::new();

    match resp.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response from github when getting repo list: {}", e),
    }
    // If needed in the future, pagination info is in resp.headers()

    repo_list_from_string(&buffer)
}

fn repo_list_from_string(json_str: &str) -> Result<Vec<GithubRepo>, String> {
    // This looks a bit weird due to supplying type hints to deserialize:
    let _: Vec<GithubRepo> = match serde_json::from_str(json_str) {
        Ok(v) => return Ok(v),
        Err(e) => return Err(format!("Couldn't deserialize repos from github: {}", e)),
    };
}

pub fn existing_release_pr_location(repo: &GithubRepo, token: &str) -> Option<String> {
    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let url = match Url::parse_with_params(&repo_pr_url, &[("head", "master"), ("base", "release")]) {
        Ok(new_url) => new_url,
        Err(e) => {
            println!("Couldn't create url for existing pr location: {}", e);
            return None;
        }
    };
    let client = match reqwest::Client::new() {
        Ok(new_client) => new_client,
        Err(e) => {
            println!("Couldn't create new reqwest client: {}", e);
            return None;
        }
    };
    let mut res = match client
              .get(url)
              .header(UserAgent(USERAGENT.to_string()))
              .header(Authorization(format!("token {}", token)))
              .send() {
        Ok(response) => response,
        Err(e) => {
            println!("Error in request to github for existing PR location: {}", e);
            return None;
        }
    };

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

// Try to create the release PR and return the URL of it:
pub fn create_release_pull_request(repo: &GithubRepo, token: &str) -> Result<String, String> {
    let mut pr_body = HashMap::new();
    pr_body.insert("title", "automated release partay");
    pr_body.insert("head", "master");
    pr_body.insert("base", "release");

    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let client = match reqwest::Client::new() {
        Ok(new_client) => new_client,
        Err(e) => return Err(format!("Couldn't create new reqwest client: {}", e)),
    };
    let mut res = match client
              .post(&repo_pr_url)
              .json(&pr_body)
              .header(UserAgent(USERAGENT.to_string()))
              .header(Authorization(format!("token {}", token)))
              .send() {
        Ok(response) => response,
        Err(e) => return Err(format!("Error in request to github creating new PR: {}", e)),
    };

    if res.status().is_success() {
        let mut buffer = String::new();
        match res.read_to_string(&mut buffer) {
            Ok(_) => (),
            Err(e) => println!("error reading response after creating new release PR for {}: {}", repo.name, e),
        }
        let pull_req: GithubPullRequest = match serde_json::from_str(&buffer) {
            Ok(v) => v,
            Err(e) => return Err(format!("Couldn't deserialize create pull req response for {}: {}", repo.name, e)),
        };
        return Ok(pull_req.html_url);
    }
    // 422 unprocessable means it's there already
    // 422 unprocessable also means the branch is up to date

    Err("Release branch already up to date?".to_string())
}
