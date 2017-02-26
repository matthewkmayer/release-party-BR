extern crate serde_json;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;

use std::env;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct GithubRepo {
    id: i32,
    name: String,
    url: String
}

fn repo_list_from_string(json_str: String) -> Result<Vec<GithubRepo>, String> {
    let repo_list : Vec<GithubRepo> = match serde_json::from_str(&json_str)  {
        Ok(v) => v,
        Err(e) => return Err(format!("Couldn't deserialize it: {}", e)),
    };

    println!("repo_list is {:?}", repo_list);
    return Ok(repo_list);
}

fn main() {
    println!("Hello, world!");
    let key = "RP_GITHUBTOKEN";
    match env::var(key) {
        Ok(val) => println!("{}: {:?}", key, val),
        Err(e) => println!("Couldn't find {}: {}", key, e),
    }

    let mut resp = reqwest::get("https://api.github.com/users/matthewkmayer/repos").unwrap();
    // println!("resp is: {:?}", resp);
    let mut buffer = String::new();

    match resp.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response: {}", e),
    }
    // println!("request body: {}", buffer);

    let foo = repo_list_from_string(buffer).unwrap();

    
}
