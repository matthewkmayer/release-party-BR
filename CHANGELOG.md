# Release-party changes

## [Unreleased]

## [0.6.0] - 2019-06-28

- Support for looking the home dir for ignoredrepos.toml
- Add a fancy progress bar
- Provide a useful error if GitHub sends unexpected shapes

## [0.5.0] - 2019-02-22
- Put version of `release-party` used in each release PR
- Added image showing what token permissions are required

## [0.4.0] - 2019-01-19
- Added CHANGELOG
- Improved error messages
- Add bulleted list of PRs included in each release PR

## [0.3.1] - 2017-09-21

### Added
- Fixed a call to GitHub that didn't use the throttler to avoid hitting API limits

## [0.3.0] - 2017-09-20

### Added
- Handle paging of repo results from GitHub

### Changed
- Only need to provide the GitHub org instead of the entire org's API URL

### Removed
- Parallelism via rayon to comply with GitHub API guidelines
