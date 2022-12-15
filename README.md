# Substrace: Experimental Static Analysis for Substrate Projects

[Substrace](https://github.com/kaiserkarel/substrace) is a tool for linting substrate projects for specific gotchas and other errors.

# Installing
Install through `cargo install substrace`. Run using `cargo substrace`.

# Content
Currently the following lints are used:
- `missing_security_doc`: checks for the presence `Security` headers on storage maps using suspicious hashing functions, such as `Twox64Concat`. 
- `panics`: ensures that clippy has been properly configured to avoid panics in code.
- `missing_transactional`: ensures that extrinsics are marked with the `#[transactional]` label
- `extrinsics_must_be_tagged`: ensures that extrinsics are marked with the `#[pallet::call_index(...)]` label

More lints will be added in the future, initially targetting checks that ensure storage consistencies. Currently in the works:
- `storage_iter_insert`: checks that storage isn't simultaniously being mutated whilst iteration is active.
- `enable_singlepass_benchmarks`: checks that benchmarks are run when calling `cargo test`
- `xcm_config_check`: checks that the xcm config does not make the same mistake as the Karura bug
