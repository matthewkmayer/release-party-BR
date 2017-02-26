# release-party-BR
Release party automation

### Game plan
1. Be able to talk to github API.
2. Get list of repos from an org.
3. Check each repo to see if `release` branch is behind `master`.
4. If a repo is, create new PR from `master` to `release`.
5. Show a list of all repos and their status: up to date or which PR brings `release` up to speed.

### Future work
1. Make which org configurable.
2. Accept list of repos to ignore.
3. Links to PRs for releasing.
4. Put everything behind a RESTful API to allow fancy web interface.
5. Talk to CI to get status of PR?