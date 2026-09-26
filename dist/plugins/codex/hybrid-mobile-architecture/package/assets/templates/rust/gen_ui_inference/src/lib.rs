// TJ-ARCH-MOB-001 compliant
//! gen_ui_inference (L2) — local-inference engines behind the
//! `gen_ui_types::inference::InferenceProvider` trait. Per-lane engines
//! (see versions.toml [inference]): LiteRT-LM on Android, pinned llama.cpp
//! (`llama-cpp-2`) where still certified, WebLLM on web, and mistral.rs as an
//! optional experiment.
//!
//! Callers (gen_ui_agent, the platform leaves) depend on the trait only — never
//! on an engine crate — so adding or swapping a lane never ripples past here.

// `local-mistral` is wired in Cargo.toml but ships no module: mistral.rs has no
// documented mobile deployment path and does not build for wasm32, so the
// scaffold leaves it as a seam rather than a dependency. Add src/mistral.rs and
// the target-gated `dep:mistralrs` entry to enable it, then restore the
// `pub mod mistral;` / `pub use mistral::MistralEngine;` pair here behind the
// same cfg the other lanes use.

#[cfg(feature = "local-llama")]
pub mod llama;

#[cfg(any(
    feature = "local-llama",
    feature = "local-litert-lm",
    feature = "local-mlx",
    feature = "local-mlxc"
))]
pub mod catalog;

/// Device-memory preflight shared across mobile engines (jetsam gate).
#[cfg(any(feature = "local-llama", feature = "local-mlx", feature = "local-mlxc"))]
pub(crate) mod memory;

/// Tool-schema prompt injection shared by engines without a native tool API.
#[cfg(any(feature = "local-llama", feature = "local-mlx"))]
pub(crate) mod prompt;

/// Multi-file MLX model download/verify, shared by the iOS and macOS MLX lanes.
#[cfg(any(feature = "local-mlx", feature = "local-mlxc"))]
pub(crate) mod mlx_download;

/// iOS MLX-Swift lane. The engine is host-testable via a mock bridge; the FFI
/// vtable that binds the real Swift bridge is iOS-only.
#[cfg(feature = "local-mlx")]
pub mod mlx;

#[cfg(feature = "local-mlx")]
pub use mlx::MlxEngine;

#[cfg(all(feature = "local-mlx", target_os = "ios"))]
pub mod ios_mlx_ffi;

/// macOS MLX lane: in-process inference via the mlex crate (mlx-c). Compiled
/// only on macOS builds — the `mlex` dep is target-gated — so the module is
/// gated on both feature and target.
#[cfg(all(feature = "local-mlxc", target_os = "macos"))]
pub mod mlxc;

#[cfg(all(feature = "local-mlxc", target_os = "macos"))]
pub use mlxc::MlxcEngine;

#[cfg(all(feature = "local-litert-lm", target_os = "android"))]
pub mod android_litert_jni;

#[cfg(feature = "local-llama")]
pub use llama::LlamaCppEngine;

#[cfg(feature = "local-litert-lm")]
pub mod litert_lm;

#[cfg(feature = "local-litert-lm")]
pub use litert_lm::LiteRtLmEngine;
