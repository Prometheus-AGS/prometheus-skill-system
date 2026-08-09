# nanochat Tokenizer Pipeline Walkthrough

## Overview

`nanochat` is a minimal GPT-style chat model designed to run locally on
commodity hardware. Its tokenizer pipeline follows the rustbpe → tiktoken
pattern: Python training at build time, Rust inference at runtime.

This document annotates the pattern so you can replicate it in your own project.

## Step 1: Corpus Preparation

```
data/
├── train.txt          # primary training corpus (shuffled)
├── val.txt            # held-out validation split (10%)
└── metadata.json      # {source, token_count, date}
```

The corpus is UTF-8 text, one document per line. `nanochat` uses a mix of
CommonCrawl snippets and synthetic chat turns. Keep document boundaries as
newlines so the regex pre-tokenizer (which splits on whitespace) does not merge
across documents.

## Step 2: Training

```python
# scripts/train_tokenizer.py (simplified nanochat version)
import rustbpe, pathlib, base64, json

VOCAB_SIZE = 8192
SPECIAL_TOKENS = {
    "<|endoftext|>": 8192,
    "<|im_start|>":  8193,
    "<|im_end|>":    8194,
}

tokenizer = rustbpe.Tokenizer()
corpus = pathlib.Path("data/train.txt").open()

tokenizer.train_from_iterator(corpus, vocab_size=VOCAB_SIZE, buffer_size=16384)

# Sanity check: compression ratio
sample = pathlib.Path("data/val.txt").read_text()[:10_000]
ratio  = len(tokenizer.encode(sample)) / len(sample.split())
assert ratio < 1.6, f"Poor compression: {ratio:.2f} tokens/word"

# Export
ranks = {bytes(tok): rank for tok, rank in tokenizer.get_mergeable_ranks()}
lines = [
    f"{base64.b64encode(tok).decode()} {rank}\n"
    for tok, rank in sorted(ranks.items(), key=lambda x: x[1])
]
pathlib.Path("tokenizers/nanochat.tiktoken").write_text("".join(lines))

# Save pattern and special tokens for later tiktoken wiring
meta = {
    "pattern": tokenizer.get_pattern(),
    "vocab_size": tokenizer.vocab_size,
    "special_tokens": SPECIAL_TOKENS,
}
pathlib.Path("tokenizers/meta.json").write_text(json.dumps(meta, indent=2))
print(f"Trained {tokenizer.vocab_size} tokens, ratio={ratio:.2f}")
```

## Step 3: Validating the Export

Before the Rust build reads the tokenizer, validate the round-trip in Python:

```python
import tiktoken, json, pathlib

meta  = json.loads(pathlib.Path("tokenizers/meta.json").read_text())
lines = pathlib.Path("tokenizers/nanochat.tiktoken").read_text().splitlines()
ranks = {}
for line in lines:
    parts = line.split()
    import base64
    ranks[base64.b64decode(parts[0])] = int(parts[1])

enc = tiktoken.Encoding(
    name="nanochat",
    pat_str=meta["pattern"],
    mergeable_ranks=ranks,
    special_tokens=meta["special_tokens"],
)

for text in ["Hello world", "fn main() {}", "<|im_start|>user\nhi<|im_end|>"]:
    decoded = enc.decode(enc.encode(text))
    assert decoded == text, f"Round-trip failed: {text!r} -> {decoded!r}"

print("Validation passed")
```

## Step 4: Rust Runtime (agent-tokenizer)

The `agent-tokenizer` crate (from the `native-agent` skill) wraps
`tiktoken-rs` and exposes a stable API to the rest of the agent:

```toml
# Cargo.toml
[dependencies]
tiktoken-rs = "0.5"
anyhow      = "1"
```

```rust
// src/tokenizer.rs — generated from agent_tokenizer.rs.tera
use tiktoken_rs::CoreBPE;

pub struct Tokenizer { inner: CoreBPE }

impl Tokenizer {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let bpe = tiktoken_rs::load_tiktoken_bpe(path)?;
        Ok(Self { inner: bpe })
    }

    pub fn encode(&self, text: &str) -> Vec<usize> {
        self.inner.encode_ordinary(text)
    }

    pub fn count_tokens(&self, text: &str) -> usize {
        self.encode(text).len()
    }

    pub fn truncate_to(&self, text: &str, max_tokens: usize) -> String {
        let ids = self.encode(text);
        if ids.len() <= max_tokens {
            return text.to_string();
        }
        self.inner.decode(ids[..max_tokens].to_vec()).unwrap_or_default()
    }
}
```

## Step 5: Wiring in the Training Loop

In `nanochat`'s training loop, the tokenizer is used to convert raw text into
token ID sequences for the model:

```rust
let tok = Tokenizer::load("tokenizers/nanochat.tiktoken")?;

// Prompt-budget enforcement before sending to model
let prompt = build_prompt(&conversation);
let token_count = tok.count_tokens(&prompt);
if token_count > MAX_CONTEXT {
    let (truncated, _) = tok.truncate_to(&prompt, MAX_CONTEXT);
    // use truncated
}
```

## Key Invariants

1. **Pattern must match between training and inference.** The `pat_str` passed
   to `tiktoken.Encoding` must be `tokenizer.get_pattern()` — not a hardcoded
   string. Mismatches cause silent tokenization divergence.

2. **Special tokens are not trained; they are injected.** `rustbpe` only trains
   merge tokens. Special tokens are IDs above `vocab_size` and must be handled
   by the inference library (`tiktoken` has `allowed_special` / `disallowed_special`
   parameters for this).

3. **Save `meta.json` alongside the tiktoken file.** The pattern, vocab size,
   and special token map are needed to reconstruct the `tiktoken.Encoding` in
   any validation or fine-tuning script.
