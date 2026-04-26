# Provider Environment Variables

Canonical list of provider env vars that `scripts/detect-providers.sh` scans. When any of these is set, the matching provider is considered available for the listed classes.

## Frontier-Class Providers

| Provider     | Env Var              | Notes                                              |
|--------------|----------------------|----------------------------------------------------|
| Anthropic    | `ANTHROPIC_API_KEY`  | Sonnet 4.6 / Opus → frontier; Haiku → medium       |
| OpenAI       | `OPENAI_API_KEY`     | GPT-4o / o1 → frontier; GPT-4o-mini → medium/small |
| Google       | `GOOGLE_API_KEY` or `GEMINI_API_KEY` | Gemini 2.0 Pro → frontier; Flash → medium |
| AWS Bedrock  | `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` + `AWS_REGION` | Claude / Llama via Bedrock |

## Medium-Class Providers

| Provider     | Env Var              | Notes                                              |
|--------------|----------------------|----------------------------------------------------|
| Groq         | `GROQ_API_KEY`       | Llama 3.3 70B → medium; 8B → small                 |
| Together     | `TOGETHER_API_KEY`   | Mixtral 8x22B → medium; 7B variants → small        |
| Mistral      | `MISTRAL_API_KEY`    | Mistral Large → frontier/medium; Small → small     |
| Cohere       | `COHERE_API_KEY`     | Command-R+ → medium; Command-R → small             |
| Fireworks    | `FIREWORKS_API_KEY`  | Routes by model family                             |

## Small-Class Providers (local / cheap)

| Provider     | Env Var              | Notes                                              |
|--------------|----------------------|----------------------------------------------------|
| Ollama       | `OLLAMA_HOST`        | Default `http://localhost:11434`                   |
| vLLM         | `VLLM_BASE_URL`      | Self-hosted OpenAI-compatible endpoint             |
| LM Studio    | `LMSTUDIO_BASE_URL`  | OpenAI-compatible local                            |
| llama.cpp    | `LLAMA_CPP_SERVER`   | OpenAI-compatible local                            |
| OpenRouter   | `OPENROUTER_API_KEY` | Multi-tier — class depends on selected model       |

## Class Assignment Rules

A provider is mapped to a class based on the *cheapest available model* that meets the class's cognitive requirement:

- **frontier** = top-tier reasoning model from a frontier provider (Sonnet 4.6, GPT-4o, Gemini 2.0 Pro, Mistral Large)
- **medium** = mid-tier model with strong instruction following (Haiku 4.5, Llama 3.3 70B, Mixtral, Command-R+)
- **small** = local or fast hosted model for mechanical work (Llama 3 8B, Phi, Mistral 7B, Qwen 2.5 7B/9B)

`scripts/detect-providers.sh` does NOT pick the specific model — it only reports which classes have *some* viable provider. The configure prompt picks the concrete model and writes the alias table.

## Override

The user can pin specific models via `~/.config/liter-llm/config.toml`:

```toml
[aliases]
small    = "ollama/qwen2.5:7b"
medium   = "groq/llama-3.3-70b-versatile"
frontier = "anthropic/claude-sonnet-4-6"
```

Pinned aliases take precedence over the auto-detect logic.
