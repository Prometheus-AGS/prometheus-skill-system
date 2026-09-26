---
name: local-inference-lanes
description: ALWAYS invoke before adding, selecting, or debugging a local/on-device inference engine, or before writing any code that picks where a chat turn runs — the engine is a per-DEVICE choice (Android LiteRT-LM, iOS/macOS MLX, desktop llama.cpp, web WebLLM), never one "mobile" engine, and lanes are cloud/local/uar. Also invoke before writing model download, model verification, or RAM-budget code. Triggers on local inference, on-device model, LiteRT-LM, litertlm, MLX, mlx-swift, mlex, llama.cpp, llama-cpp-2, GGUF, WebLLM, InferenceProvider, model catalog, model download, model manifest, quantization, Q4_K_M, lane, cloud lane, local lane, uar lane, jetsam, OOM on model load, engine selection, BYOK.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Prefer simple, surgical, strongly typed changes; preserve strict
> layering and verify dependency versions. When installed in a project, also
> obey that project's `AGENT_BASE_RULES.md`; this skill remains self-contained.

# Local Inference Lanes

## The rule

**The engine is a per-DEVICE choice. The lane is a per-TURN choice. They are
different things, and neither is "mobile vs desktop".**

Every engine sits behind one `InferenceProvider` seam, so surface code never
branches on engine. Callers depend on the trait; only this crate depends on an
engine.

## Per-device engines

| Surface | Engine | Why this one |
|---|---|---|
| Desktop | `llama-cpp-2` | Pinned, checksummed GGUF. Zero-config reference default. |
| Android | **LiteRT-LM** | Universal correctness baseline. |
| iOS | **MLX-Swift** | Reaches Metal without the llama.cpp build surface. |
| macOS | **MLX-C** (`mlex`) | In-process, Apple's mlx-c C API — no Python runtime. |
| Web | WebLLM | MLC WebGPU lane; zero-key browser default. |

Current values live in `versions.toml` `[inference]`. Read them there — do not
copy them into code.

### Why Android is not llama.cpp

This is the part that gets assumed wrong. llama.cpp *builds* for Android, so it
looks like the obvious single mobile engine. In practice its GPU path
(OpenCL/Adreno) proved **device-specific**: it works on the phone you tested and
fails on the next one, which is the worst possible failure shape for a
correctness baseline. LiteRT-LM runs everywhere on CPU and is therefore the
Android default; llama.cpp stays quarantined out of Android production.

**There is no "mobile engine."** A `[inference] mobile = ...` key, or a
`cfg(mobile)` branch selecting an engine, is the bug this table exists to
prevent.

## The three lanes

A lane is where a turn runs. There are exactly three:

| Lane | Meaning | Notes |
|---|---|---|
| `cloud` | BYOK remote provider | Cannot be the zero-config default — an unconfigured install has no key. |
| `local` | On-device via `InferenceProvider` | The row carries `model_id` and **no** `provider_id`; there is no remote provider to name. |
| `uar` | Universal Agent Runtime | HTTP/SSE on desktop and web. Mobile must use **embedded-library** mode, never a local sidecar. |

Parse lanes through the typed `Lane` enum. **Reject an unknown lane loudly** —
never fall back to a default. A silent fallback runs the turn somewhere the user
did not choose, and on the `cloud` lane that means data leaving the device.

## Half Rust, half native — and the halves agree by name

Android and iOS lanes are split across languages:

- **Rust owns** catalog, download, verification, RAM preflight, tool policy, and
  event normalization.
- **The native bridge owns** only the vendor SDK's generation call
  (`<App>LiteRtLmBridge.kt`, `<App>MlxBridge.swift`).

They are bound by **symbol name** — the JNI fully-qualified class path, and the
Swift `@_silgen_name` FFI symbol. Rename one without the other and you get a
build that succeeds and a lane that cannot find its bridge **at runtime, on
device**.

Two consequences:

- The JNI package path must be a legal Java identifier. `com.example.my-app` is
  not — hyphens are illegal in Java/Kotlin package names. It compiles in Rust as
  a plain string and fails to resolve the class on device.
- A Swift file on disk that is not in `project.pbxproj` is not compiled. The
  failure is a missing symbol at link time, not a missing file.

## Model download discipline

An interrupted or corrupted model is indistinguishable from a broken engine
unless the download layer is strict:

- **Resumable.** Multi-GB files over mobile networks; restarting from zero is not
  a recovery strategy.
- **Revision-pinned.** Pin the repo revision, not just the filename. "Latest"
  silently changes weights under a cached path.
- **SHA-256 verified** before first load.
- **Small files first.** Download config/tokenizer before weights so a bad
  revision fails in seconds instead of after gigabytes.
- **Never cache a partial file under the final path.** A resumed run must not
  mistake it for complete.

## RAM preflight is mandatory, not defensive

On iOS an over-budget model load does not throw — **jetsam kills the process
outright**. There is no exception to catch and no log to read afterwards.

So the size gate runs **before** the load, never around it. Compute a budget from
physical memory, keep the fraction conservative (the OS and the app's own
allocations draw on the same pool), and refuse the load with a real error the UI
can render.

## Red flags — stop if you are about to write these

| You're writing | Why it's wrong |
|---|---|
| A single "mobile" engine constant or `cfg(mobile)` engine branch | Android and iOS use different engines. This is the core mistake. |
| `llama.cpp` as the Android production engine | Device-specific GPU path; quarantined for exactly this reason. |
| A lane string parsed with a `_ => default` arm | Silently runs the turn on a lane the user did not pick. |
| A fourth lane string | Update the `Lane` enum and both surfaces, or don't add it. |
| Engine types leaking past `InferenceProvider` | Every caller then has to know which engine is loaded; swapping a lane ripples through the app. |
| Model load with no memory check | On iOS this is a process kill, not an error. |
| A hyphen in the JNI package path | Not a legal Java identifier; fails at class resolution on device. |
| "It compiles, so the lane works" | Local lanes fail at model load, after every host check passes. See `hybrid-runtime-verification`. |

## Verification

Compiling proves nothing about a local lane. The device gates are the real
evidence:

```bash
node scripts/android/build.mjs
node scripts/android/verify-native-inference-gates.mjs   # arm64-only APK, required .so present
node scripts/android/verify-device-runtime-gates.mjs     # no JNI/dlopen failure in logcat
```

Per-lane host checks are still worth running, because a feature-gated module that
nobody compiles rots:

```bash
cargo check -p gen_ui_inference --features local-litert-lm
cargo check -p gen_ui_inference --features local-mlx
```

## Scope note

This skill owns **engine selection, lane routing, model acquisition, and the
native bridge contract**. It does not own how generated output is rendered
(`content-block-ui`), retrieval (`client-rag`), or whether a build is shippable
(`hybrid-runtime-verification`).
