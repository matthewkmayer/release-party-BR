extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct GithubRepo {
    id: i32,
    name: String,
    url: String
}

pub fn repo_list_from_string(json_str: String) -> Result<Vec<GithubRepo>, String> {
    let repo_list : Vec<GithubRepo> = match serde_json::from_str(&json_str)  {
        Ok(v) => v,
        Err(e) => return Err(format!("Couldn't deserialize it: {}", e)),
    };

    println!("repo_list is {:?}", repo_list);
    return Ok(repo_list);
}