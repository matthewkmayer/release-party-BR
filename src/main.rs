extern crate reqwest;

use std::env;
use std::io::Read;

fn main() {
    println!("Hello, world!");
    let key = "RP_GITHUBTOKEN";
    match env::var(key) {
        Ok(val) => println!("{}: {:?}", key, val),
        Err(e) => println!("Couldn't find {}: {}", key, e),
    }

    let mut resp = reqwest::get("https://api.github.com/users/matthewkmayer/repos").unwrap();
    println!("resp is: {:?}", resp);
    let mut buffer = String::new();

    match resp.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => println!("error reading response: {}", e),
    }
    println!("request body: {}", buffer);
}
