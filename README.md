# Substrace: Experimental Static Analysis for Substrate Projects

[Substrace](https://github.com/kaiserkarel/substrace) is a tool for linting substrate projects for specific gotchas and other errors.

# Installing

First install [dylint](https://github.com/trailofbits/dylint), then add the following to your workspace's cargo.toml:

```toml
[workspace.metadata.dylint]
libraries = [
    { git = "https://github.com/kaiserkarel/substrace" },
]
```

Now running `cargo dylint substrace --workspace` will download the linter and check your crates. Currently the following lints are used:

- `panics`: ensures that clippy has been properly configured to avoid panics in code.
- `storage_iter_insert`: checks that storage isn't simultaniously being mutated whilst iteration is active.
- `missing_security_doc`: checks for the presence `Security` headers on storage maps using suspicious hashing functions, such as `Twox64Concat`. 

More lints will be added in the future, initially targetting checks that ensure storage consistencies.
