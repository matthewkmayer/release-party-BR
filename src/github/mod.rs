extern crate reqwest;
extern crate serde_json;

use std::io::Read;


#[derive(Serialize, Deserialize, Debug)]
pub struct GithubRepo {
    id: i32,
    pub name: String,
    pub url: String
}

pub fn get_repos_at(repos_url: &str) -> Result<Vec<GithubRepo>, String> {
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

// Try to create the release PR and return the URL of it:
pub fn create_release_pull_request(repo: &GithubRepo) -> Result<String, String> {
    if repo.name == "calagator" {
        return Err("already up to date".to_string());
    }
    Ok("https://github.whatever/reponame/prs/1".to_string())
}