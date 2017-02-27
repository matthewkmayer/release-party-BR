extern crate reqwest;
extern crate serde_json;

use self::reqwest::header::{Authorization, UserAgent};

use std::io::Read;
use std::collections::HashMap;

static USERAGENT: &'static str = "release-party-br";

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubRepo {
    id: i32,
    pub name: String,
    pub url: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubPullRequest {
    id: i32,
    pub url: String,
    pub html_url: String
}

pub fn get_repos_at(repos_url: &str, token: &str) -> Result<Vec<GithubRepo>, String> {
    let mut resp = reqwest::get(repos_url).unwrap();
    let mut buffer = String::new();

    match resp.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response: {}", e),
    }

    return repo_list_from_string(buffer);
}

fn repo_list_from_string(json_str: String) -> Result<Vec<GithubRepo>, String> {
    // This looks a bit weird due to supplying type hints to deserialize:
    let _ : Vec<GithubRepo> = match serde_json::from_str(&json_str)  {
        Ok(v) => return Ok(v),
        Err(e) => return Err(format!("Couldn't deserialize it: {}", e)),
    };
}

pub fn existing_release_pr_location(repo: &GithubRepo, token: &str) -> Option<String> {
    let mut pr_check_body = HashMap::new();
    pr_check_body.insert("head", "master");
    pr_check_body.insert("base", "release");

    let repo_pr_url = format!("{}/{}", repo.url, "pulls");
    let client = reqwest::Client::new().unwrap();
    let mut res = client.get(&repo_pr_url)
        .json(&pr_check_body)
        .header(UserAgent(USERAGENT.to_string()))
        .header(Authorization(format!("token {}", token)))
        .send()
        .unwrap();

    let mut buffer = String::new();

    match res.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response: {}", e),
    }

    let pull_reqs : Vec<GithubPullRequest> = match serde_json::from_str(&buffer)  {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };

    if pull_reqs.len() > 0 {
        return Some(pull_reqs[0].html_url.clone());
    }

    return None;
}

// Try to create the release PR and return the URL of it:
pub fn create_release_pull_request(repo: &GithubRepo, token: &str) -> Result<String, String> {
    if repo.name != "dot-net-web-api-experiment" {
        return Err("already up to date".to_string());
    }
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
            Err(e) => println!("error reading response: {}", e),
        }
        let pull_req : GithubPullRequest = match serde_json::from_str(&buffer)  {
            Ok(v) => v,
            Err(e) => return Err(format!("Couldn't deserialize pull req response: {}", e)),
        };
        return Ok(pull_req.html_url);
    }
    // 422 unprocessable means it's there already
    // 422 unprocessable also means the branch is up to date

    Err("Didn't get successful response: release branch up to date already?".to_string())
}