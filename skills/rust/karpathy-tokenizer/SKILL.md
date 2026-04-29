---
name: karpathy-tokenizer
description: Train GPT-style BPE tokenizers with rustbpe (Karpathy), export to tiktoken for fast Rust inference, and integrate with agent-tokenizer for prompt-budget enforcement.
license: MIT
version: "1.0.0"
authors: ["prometheus-skill-pack"]
compatibility: "rustbpe==0.1.0, tiktoken>=0.7, Python>=3.10"
---

# Karpathy Tokenizer

## When to Use

- You need to train a custom BPE tokenizer on domain-specific text (code, chat logs, medical records).
- You want fast Rust inference via `tiktoken` after training in Python with `rustbpe`.
- You are building a `nanochat`-style agent that needs context-budget enforcement from an `agent-tokenizer` crate.
- You want the simplest possible path from raw text → tiktoken file → Rust encode/decode.

## Architecture

```
raw text corpus
    │
    ▼  Python training (rustbpe — fast, parallel)
rustbpe.Tokenizer.train_from_iterator()
    │
    ▼  export
tokenizer.tiktoken          ← used at Rust runtime
    │
    ▼  Rust runtime (tiktoken-rs or agent-tokenizer wrapper)
encode / decode / count_tokens
```

`rustbpe` is the **training** side. `tiktoken` is the **inference** side.
Do not call `rustbpe.encode()` in production hot paths — it exists for validation only.

## Installation

```bash
# Python training environment
pip install rustbpe tiktoken

# Rust inference (add to Cargo.toml)
# tiktoken-rs = "0.5"
```

## Training a Tokenizer

```python
import rustbpe

tokenizer = rustbpe.Tokenizer()

# Train from a list of strings (or any iterator)
tokenizer.train_from_iterator(
    open("data/corpus.txt"),   # file iterator yields lines
    vocab_size=8192,           # 256 byte tokens + 7936 merges
    buffer_size=8192,          # lines buffered per rayon batch
    pattern=None,              # None → GPT-4 regex (recommended)
)

print(tokenizer.vocab_size)   # 8192
```

### Custom regex pattern

The default is the GPT-4 pattern, which handles contractions, unicode letters,
numbers, punctuation, and whitespace correctly. Only override if you have a
specific tokenization requirement:

```python
# Domain-specific override (e.g., code-only, no unicode splitting)
tokenizer.train_from_iterator(
    texts,
    vocab_size=4096,
    pattern=r"[a-zA-Z_][a-zA-Z0-9_]*|[0-9]+|[^\s]|\s+",
)
```

### Pitfall: vocab_size must exceed 256

The first 256 tokens are byte-level (0x00–0xff). `vocab_size` counts total
tokens including those 256. A `vocab_size=256` produces zero merges.

```python
# WRONG — no merges, useless tokenizer
tokenizer.train_from_iterator(texts, vocab_size=256)

# CORRECT — 256 byte tokens + 3840 merge tokens
tokenizer.train_from_iterator(texts, vocab_size=4096)
```

## Export to tiktoken

This is the primary workflow. Train once, export once, use tiktoken forever:

```python
import rustbpe, tiktoken, json, pathlib

tokenizer = rustbpe.Tokenizer()
tokenizer.train_from_iterator(open("data/corpus.txt"), vocab_size=8192)

# Build tiktoken-compatible ranks
mergeable_ranks = {
    bytes(token): rank
    for token, rank in tokenizer.get_mergeable_ranks()
}

# Wire into tiktoken Encoding object
enc = tiktoken.Encoding(
    name="my_model",
    pat_str=tokenizer.get_pattern(),      # must match training pattern
    mergeable_ranks=mergeable_ranks,
    special_tokens={},
)

# Validate round-trip before saving
sample = "Hello, world! This is a test."
assert enc.decode(enc.encode(sample)) == sample

# Save the tiktoken file (one line per token: base64(bytes) rank)
import base64
lines = [
    f"{base64.b64encode(token).decode()} {rank}\n"
    for token, rank in sorted(mergeable_ranks.items(), key=lambda x: x[1])
]
pathlib.Path("tokenizer.tiktoken").write_text("".join(lines))
```

### Pitfall: special tokens

`rustbpe` does not add special tokens (`<|endoftext|>`, `<|im_start|>`, etc.)
during training. Add them after export:

```python
special_tokens = {
    "<|endoftext|>": 8192,
    "<|im_start|>":  8193,
    "<|im_end|>":    8194,
}

enc = tiktoken.Encoding(
    name="my_model",
    pat_str=tokenizer.get_pattern(),
    mergeable_ranks=mergeable_ranks,
    special_tokens=special_tokens,  # added here, not during training
)
```

## Encoding and Decoding (Python validation only)

```python
ids  = tokenizer.encode("Hello world")      # List[int]
text = tokenizer.decode(ids)                 # str  — round-trip
all  = tokenizer.batch_encode(["a", "b"])    # List[List[int]] — parallel

# Pitfall: decode can raise if token IDs are out of vocab range
# Always validate with assert decode(encode(x)) == x before shipping
```

## Rust Inference via tiktoken-rs

```rust
// Cargo.toml: tiktoken-rs = "0.5"
use tiktoken_rs::CoreBPE;
use std::fs;

fn load_tokenizer(path: &str) -> CoreBPE {
    // tiktoken-rs can load from a .tiktoken file directly
    tiktoken_rs::load_tiktoken_bpe(path).expect("load tokenizer")
}

fn main() {
    let enc = load_tokenizer("tokenizer.tiktoken");
    let ids  = enc.encode_ordinary("Hello, world!");
    let text = enc.decode(ids).expect("decode");
    assert_eq!(text, "Hello, world!");
}
```

## Integration with agent-tokenizer

The `agent-tokenizer` crate template (from `native-agent`) wraps the runtime
tokenizer for prompt-budget enforcement. Point it at the exported `.tiktoken`
file:

```bash
# Set env var for agent runtime
export MY_AGENT_TOKENIZER=tokenizers/my_model.tiktoken
```

The `agent-tokenizer` `Tokenizer::load()` reads the file, and
`count_tokens()` / `truncate_to()` enforce context limits.

See `references/nanochat-walkthrough.md` for how `nanochat` wires this end-to-end.

## Complete Training Script

See `templates/train_tokenizer.py.tera` for a ready-to-run CLI that trains,
validates, and exports in one command:

```bash
python train_tokenizer.py --corpus data/ --vocab-size 8192 --output tokenizers/
```

## See Also

- [rustbpe vs tiktoken](references/rustbpe-vs-tiktoken.md) — when to train vs load pretrained
- [nanochat walkthrough](references/nanochat-walkthrough.md) — end-to-end pipeline tour
- [train_tokenizer.py.tera](templates/train_tokenizer.py.tera) — CLI training script
- [load_tokenizer.rs.tera](templates/load_tokenizer.rs.tera) — Rust runtime loader
