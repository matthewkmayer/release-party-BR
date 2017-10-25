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

Prebuilt binaries for Linux and OSX are available on [the releases page](https://github.com/matthewkmayer/release-party-BR/releases).

## Running

#### Required information

* `RP_GITHUBTOKEN` - environment variable for a personal access token to Github
* `--org` - GitHub organization name

#### Optional behavior

* `dry-run` - See what PRs would be created: `RP_GITHUBTOKEN=your_personal_token_here cargo run -- --org "ORGHERE" --dry-run`
* repo ignore list - The `ignoredrepos.toml` file can contain a list of repositories to ignore.  See [ignoredrepos.toml](ignoredrepos.toml) for an example.

#### Running on OSX

`RP_GITHUBTOKEN=your_personal_token_here ./release-party-br-darwin-amd64 --org "ORGHERE"`

#### Running on Linux

`RP_GITHUBTOKEN=your_personal_token_here ./release-party-br-linux-amd64 --org "ORGHERE"`