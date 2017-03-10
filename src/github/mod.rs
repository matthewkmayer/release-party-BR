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

pub fn get_repos_at(repos_url: &str, token: &str) -> Result<Vec<GithubRepo>, String> {
    let url = Url::parse_with_params(repos_url, &[("per_page", "75")]).unwrap();
    println!("Getting repos at {}", url);
    let client = reqwest::Client::new().unwrap();
    let mut resp = client.get(url)
        .header(UserAgent(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
        .unwrap();
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
    let url = Url::parse_with_params(&repo_pr_url, &[("head", "master"), ("base", "release")]).unwrap();
    let client = reqwest::Client::new().unwrap();
    let mut res = client.get(url)
        .header(UserAgent(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
        .unwrap();

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
    let client = reqwest::Client::new().unwrap();
    let mut res = client.post(&repo_pr_url)
        .json(&pr_body)
        .header(UserAgent(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
        .unwrap();

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
