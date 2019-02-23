# release-party-br

Release party automation.

Designed to automate creating pull requests for releasing to production, release-party-br looks for repos in an 
organization and creates pull requests from `master` to `release` branch on each repo.  Useful when there's many 
repos ready for a production release.

<table>
    <tr>
        <td><strong>Linux / OS X</strong></td>
        <td><a href="https://travis-ci.org/matthewkmayer/release-party-BR" title="Travis Build Status"><img src="https://travis-ci.org/matthewkmayer/release-party-BR.svg?branch=master" alt="travis-badge"></img></a></td>
    </tr>
    <tr>
        <td><strong>Windows</strong></td>
        <td><a href="https://ci.appveyor.com/project/matthewkmayer/release-party-br" title="Appveyor Build Status"><img src="https://ci.appveyor.com/api/projects/status/gkiqfanbhjrhhh8v/branch/master?svg=true" alt="appveyor-badge"></img></a></td>
    </tr>
</table>

## Acquiring

Prebuilt binaries for Linux and OSX are available on [the releases page](https://github.com/matthewkmayer/release-party-BR/releases). If there's a `Permission denied` error when running the prebuilt binary, try running `chmod +x release-party-br-darwin-amd64` to make the file executable.

## Running

#### Required

* `RP_GITHUBTOKEN` - environment variable for a personal access token to Github
* `--org` - GitHub organization name

#### Optional

* `dry-run` - See what PRs would be created: `RP_GITHUBTOKEN=your_personal_token_here cargo run -- --org "ORGHERE" --dry-run`
* repo ignore list - The `ignoredrepos.toml` file can contain a list of repositories to ignore.  See [ignoredrepos.toml](ignoredrepos.toml) for an example.

#### Running on OSX

`RP_GITHUBTOKEN=your_personal_token_here ./release-party-br-darwin-amd64 --org "ORGHERE"`

#### Running on Linux

`RP_GITHUBTOKEN=your_personal_token_here ./release-party-br-linux-amd64 --org "ORGHERE"`

## Getting a token

`release-party-br` uses a GitHub token that can be created under an account's settings. It requires "full control of private repositories."

A graphical view:

---

![permissions](https://user-images.githubusercontent.com/4998189/52822542-c6b41980-3066-11e9-8bf5-c6e2e95df226.png)
