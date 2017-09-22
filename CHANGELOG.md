# Release-party changes

## [Unreleased]

(Please put an entry here in each PR)
- Added CHANGELOG

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