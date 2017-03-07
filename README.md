# release-party-br

Release party automation.

Designed to automate creating pull requests for releasing to production, release-party-br looks for repos and creates 
pull requests from `master` to `release` branch on each repo.  Useful when there's many repos ready for a production release.

## Running

### Compile and run

`RP_GITHUBTOKEN=your_personal_token_here RP_ORGURL="https://api.github.com/orgs/ORGHERE/repos" cargo run`

### Compile then run

`cargo build`

`RP_GITHUBTOKEN=your_personal_token_here RP_ORGURL="https://api.github.com/orgs/ORGHERE/repos" ./target/debug/release-party-br`

### Required environment variables

`RP_GITHUBTOKEN` - a personal access token to Github

`RP_ORGURL` - URL of the organization's repo list

### Optional: repo ignore list

The `ignoredrepos.toml` file can contain a list of repositories to ignore.  See [ignoredrepos.toml](ignoredrepos.toml) 
for an example.