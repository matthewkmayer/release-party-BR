# Bug to fix

https://github.com/matthewkmayer/release-party-BR/issues/62

```
RP_GITHUBTOKEN=ghtoken release-party-br --org "https://api.github.com/orgs/ORG-HERE/repos"
thread 'main' panicked at 'expected repos: "Couldn\'t deserialize repos from github: invalid type: map, expected a sequence at line 1 column 1"', src/libcore/result.rs:860:4
note: Run with `RUST_BACKTRACE=1` for a backtrace.
```

# Goal

TDD this fix.

## More goal

TDD a "did you mean x" response.

# Lookin' good!

```
RP_GITHUBTOKEN=foo cargo run -- --org https://api.github.com/orgs/my-org-name/repos
    Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
     Running `target/debug/release-party-br --org 'https://api.github.com/orgs/my-org-name/repos'`
thread 'main' panicked at 'Try this for the org value: Try this: my-org-name', src/main.rs:66:30
```