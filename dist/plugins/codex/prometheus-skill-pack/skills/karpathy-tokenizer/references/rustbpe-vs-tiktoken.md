# rustbpe vs tiktoken: When to Train vs Load

## The Two-Tool Model

| Concern | Tool | Language |
|---------|------|----------|
| Training a new BPE vocabulary | `rustbpe` | Python |
| Fast inference on a trained vocab | `tiktoken` | Python + Rust |
| Prompt-budget enforcement in agent | `agent-tokenizer` crate | Rust |

These tools are complementary, not competing. `rustbpe` produces the vocab file;
`tiktoken` / `agent-tokenizer` consume it.

## When to Train a Custom Tokenizer

Train a custom tokenizer when:

- Your corpus is domain-specific and the GPT-4 tokenizer splits it poorly.
  Common signals: average token length < 1.5 chars, many `Ġ`-prefixed tokens
  for domain words, unusually high token counts per sentence.
- You want a small model with a small, focused vocabulary (e.g., 4k tokens
  for a code-completion model over a single language).
- You are matching a published model's training setup exactly (e.g., reproducing
  a `nanochat` checkpoint with the same vocab).
- You have a compute budget and an existing corpus > 10 MB.

## When to Load a Pretrained Tokenizer

Prefer loading a pretrained tokenizer when:

- You are fine-tuning an existing model (GPT-4, Llama, Mistral). Using a
  different tokenizer breaks weight compatibility — the embedding matrix
  dimensions will not match.
- Your corpus is general-purpose English. `cl100k_base` (GPT-4's tokenizer,
  ~100k vocab) already handles English extremely efficiently.
- You want zero infrastructure: `tiktoken.get_encoding("cl100k_base")` works
  offline after the first download.

## Vocabulary Size Trade-offs

| Vocab Size | Typical Use | Avg Tokens/Word |
|------------|-------------|-----------------|
| 1k–4k      | Tiny domain model, toy experiments | ~1.2 |
| 8k–16k     | Small domain model, single language | ~1.1 |
| 32k–64k    | General-purpose small model | ~0.9 |
| 100k+      | Large multilingual model | ~0.7 |

Rule of thumb: larger vocab = shorter sequences = cheaper attention — but also
larger embedding tables and slower training. For agent-local tokenizers where
RAM is constrained, 4k–8k is the sweet spot.

## Compression Ratio Check

Before committing to a vocab size, measure compression ratio on a held-out sample:

```python
ratio = len(tokenizer.encode(sample_text)) / len(sample_text.split())
# tokens per word; lower = better compression
# aim for < 1.5 for domain text
```

## Special Token Budget

`rustbpe` trains on raw text with no special tokens. Reserve IDs above
`vocab_size` for them:

```python
special = {
    "<|endoftext|>": vocab_size,
    "<|im_start|>":  vocab_size + 1,
    "<|im_end|>":    vocab_size + 2,
    "<|pad|>":       vocab_size + 3,
}
```

Add a buffer of 10–20 IDs above `vocab_size` for future special tokens.

## Storage and Load Time

| Format | Size (8k vocab) | Rust load time |
|--------|----------------|----------------|
| `.tiktoken` (base64 text) | ~200 KB | < 50 ms |
| JSON ranks map | ~400 KB | ~100 ms |

Always ship the `.tiktoken` binary-line format for production.
