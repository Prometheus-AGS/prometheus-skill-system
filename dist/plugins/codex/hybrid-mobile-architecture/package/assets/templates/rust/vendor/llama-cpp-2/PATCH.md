# KnowMe `llama-cpp-2` dependency patch

- Upstream: <https://github.com/utilityai/llama-cpp-rs>
- Crate/version: `llama-cpp-2` `0.1.151`
- Registry checksum: `36ead0925a19d754d5d13761ceb0f0fe49d36f8103c3596588ac1fb8911e5223`
- Upstream source revision recorded by the crate: `5012cd28163d06f5b9407dfcdd0b36dcbe31cd2d`
- License: MIT OR Apache-2.0

The vendored files are the unmodified crates.io source except for both
`llama-cpp-sys-2` dependency declarations in `Cargo.toml` and
`Cargo.toml.orig`, which set `default-features = false`.

Without this patch, disabling the wrapper crate's `common` feature does not
disable the sys crate's default `common` feature. The sys build then includes
llama.cpp HTTP tooling on iOS and the Xcode link fails on unresolved
`httplib::Client` symbols. Remove this patch once upstream forwards the feature
correctly and a released, pinned crate has been verified on the iOS simulator.
