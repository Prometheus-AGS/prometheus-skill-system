# dspy-rs

A Rust port of **DSPy** (Declarative Self-improving Python): a framework for **programming—rather than prompting—language models**. [19]

DSPy lets you build **modular AI systems** (from simple classifiers to sophisticated RAG pipelines and agent loops) and provides **algorithms to optimize prompts and weights** so your LM produces higher-quality outputs over time. [19]

This repository aims to bring that same programming model and optimization toolkit to Rust.

## What DSPy is (Python)

DSPy is a framework for programming language models that emphasizes writing **compositional code** instead of maintaining brittle, hand-written prompts. [19]

At a high level, DSPy supports:

- Building modular AI systems (e.g., classifiers, RAG pipelines, agent loops). [19]
- Iterating quickly while DSPy optimizes prompts and weights for quality. [19]

## What we are porting to Rust (high-level parity targets)

The Rust port focuses on implementing DSPy’s major capability areas as documented in the DSPy knowledge base:

- **DSPy Programming**: programming overview, language models, **signatures**, **modules**, **adapters**, **tools**, and **MCP** (Model Context Protocol). [21]
- **DSPy Evaluation**: evaluation overview, data handling, and metrics. [21]
- **DSPy Optimization**: optimization overview and optimizers (including optimizers like **GEPA**). [21]

In other words: `dspy-rs` intends to provide the Rust equivalents of DSPy’s core abstractions (signatures/modules/adapters), tool and MCP integration, plus evaluation and optimization loops so programs can be “compiled” into better-performing pipelines. [19][21]

## Where to find more details

See the project documentation in:

- `docs/` (this repository)

Start here:

- `docs/PORTING_DSPY_FULL_PARITY_PLAN.md`

## Upstream DSPy references

- DSPy repository: https://github.com/stanfordnlp/dspy [20]
- DSPy docs: https://dspy.ai/ [19]
