# DSPy source provenance

- Upstream repository: https://github.com/GQAdonis/dspy-rs
- Upstream commit: `d4c18b059beda219e0cb53513b5b5c78ddf1b624`
- Upstream Git tree: `5af13e007be4d9835c1db263079c4bd43cececf0`
- Upstream package versions: `dspy 0.1.0` and `dspy-core 0.1.0`
- Upstream license declaration: `MIT OR Apache-2.0` in the workspace manifest, inherited by both packages. This export uses the Apache-2.0 option and includes the [Apache License 2.0](LICENSE-APACHE). The pinned tree contains no separate LICENSE or NOTICE file. `UPSTREAM-README.md` preserves the upstream README.

This export supports the Skill System's `prometheus-learn/optimize` feature. It includes the DSPy source for chat adaptation, model calls, prediction, bootstrap few-shot optimization, evaluation, signatures, primitives, and the retriever trait used by `Predict`. It also includes the `dspy-core` source tree. The Skill System CLI workspace owns dependency resolution through its `Cargo.lock`.

Prometheus Skill System modified the exported manifests and module entry points to select the optimizer's source and dependencies. The OpenAI model retains its original hosted constructor and adds an explicit endpoint factory with an optional bearer key for the Skill System's local and cloud routing. The HTTP client uses reqwest 0.13.5, and the hosted constructor loads `.env` through dotenvy. The core manifest selects yaml_serde 0.10.7 through the existing serde_yaml import name. Modified files carry a source notice. Concrete retrieval and embedding backends, alternate providers, local model backends, CLI/server programs, and the standalone upstream lockfile are outside this export.
