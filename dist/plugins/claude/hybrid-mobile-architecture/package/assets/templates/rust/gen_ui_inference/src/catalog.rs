//! Immutable mobile model artifact catalog.
//!
//! The digests are the SHA-256 values exposed by Hugging Face's immutable LFS
//! metadata (`X-Linked-Etag`).  URLs are revision-pinned so a mutable `main`
//! branch cannot silently replace a model after release.

use gen_ui_types::inference::ReasoningFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactSpec {
    pub repository: &'static str,
    pub revision: &'static str,
    pub file_name: &'static str,
    pub url: &'static str,
    pub byte_len: u64,
    pub sha256: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelArtifactSet {
    pub model_id: &'static str,
    pub model: ArtifactSpec,
    pub projector: Option<ArtifactSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobilePlatform {
    Ios,
    Android,
}

impl MobilePlatform {
    pub const fn default_model(self) -> ModelArtifactSet {
        match self {
            Self::Ios => IPHONE_QWEN35_9B,
            Self::Android => ANDROID_GEMMA4_E2B_LITERTLM,
        }
    }

    /// The platform default by id alone. iOS's default is an MLX multi-file
    /// set that `ModelArtifactSet` cannot represent, so callers that only need
    /// the id (boot's default-model selection) use this instead of
    /// `default_model()`.
    pub const fn default_model_id(self) -> &'static str {
        match self {
            Self::Ios => IOS_QWEN35_4B_MLX.model_id,
            Self::Android => ANDROID_GEMMA4_E2B_LITERTLM.model_id,
        }
    }
}

pub const IPHONE_QWEN35_9B: ModelArtifactSet = ModelArtifactSet {
    model_id: "qwen3.5-9b-q4-k-m",
    model: ArtifactSpec {
        repository: "unsloth/Qwen3.5-9B-GGUF",
        revision: "3885219b6810b007914f3a7950a8d1b469d598a5",
        file_name: "Qwen3.5-9B-Q4_K_M.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/Qwen3.5-9B-Q4_K_M.gguf",
        byte_len: 5_680_522_464,
        sha256: "03b74727a860a56338e042c4420bb3f04b2fec5734175f4cb9fa853daf52b7e8",
    },
    projector: Some(ArtifactSpec {
        repository: "unsloth/Qwen3.5-9B-GGUF",
        revision: "3885219b6810b007914f3a7950a8d1b469d598a5",
        file_name: "mmproj-F16.gguf",
        url: "https://huggingface.co/unsloth/Qwen3.5-9B-GGUF/resolve/3885219b6810b007914f3a7950a8d1b469d598a5/mmproj-F16.gguf",
        byte_len: 918_166_080,
        sha256: "f70dc3509053962b0d0d3ee8a7eacebf5d60aa560cad78254ae8698516ae029f",
    }),
};

pub const ANDROID_QWEN3_4B: ModelArtifactSet = ModelArtifactSet {
    model_id: "qwen3-4b-instruct-2507-q5-k-m",
    model: ArtifactSpec {
        repository: "bartowski/Qwen_Qwen3-4B-Instruct-2507-GGUF",
        revision: "ae44f08e1392f39c0e474af10c3ff8355c8b6688",
        file_name: "Qwen_Qwen3-4B-Instruct-2507-Q5_K_M.gguf",
        url: "https://huggingface.co/bartowski/Qwen_Qwen3-4B-Instruct-2507-GGUF/resolve/ae44f08e1392f39c0e474af10c3ff8355c8b6688/Qwen_Qwen3-4B-Instruct-2507-Q5_K_M.gguf",
        byte_len: 2_889_513_696,
        sha256: "66713ce35a58a82fe87642d4ec13425bf9b9a46800fff5c49a665ef5701439dc",
    },
    projector: None,
};

pub const ANDROID_GEMMA4_E2B_LITERTLM: ModelArtifactSet = ModelArtifactSet {
    model_id: "gemma-4-e2b-it-litertlm",
    model: ArtifactSpec {
        repository: "litert-community/gemma-4-E2B-it-litert-lm",
        revision: "9262660a1676eed6d0c477ab1a86344430854664",
        file_name: "gemma-4-E2B-it.litertlm",
        url: "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/9262660a1676eed6d0c477ab1a86344430854664/gemma-4-E2B-it.litertlm",
        byte_len: 2_588_147_712,
        sha256: "181938105e0eefd105961417e8da75903eacda102c4fce9ce90f50b97139a63c",
    },
    projector: None,
};

pub const ANDROID_GEMMA4_E2B_SM8750_LITERTLM: ModelArtifactSet = ModelArtifactSet {
    model_id: "gemma-4-e2b-it-litertlm-sm8750",
    model: ArtifactSpec {
        repository: "litert-community/gemma-4-E2B-it-litert-lm",
        revision: "9262660a1676eed6d0c477ab1a86344430854664",
        file_name: "gemma-4-E2B-it_qualcomm_sm8750.litertlm",
        url: "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/9262660a1676eed6d0c477ab1a86344430854664/gemma-4-E2B-it_qualcomm_sm8750.litertlm",
        byte_len: 3_016_294_400,
        sha256: "41dd675fbe735b6029012b5576a5716bac614fd8156de0128db4c9dff3cebd4e",
    },
    projector: None,
};

pub fn current_mobile_platform() -> Option<MobilePlatform> {
    #[cfg(target_os = "ios")]
    {
        return Some(MobilePlatform::Ios);
    }
    #[cfg(target_os = "android")]
    {
        return Some(MobilePlatform::Android);
    }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        None
    }
}

pub fn artifact_set_for_id(model_id: &str) -> Option<ModelArtifactSet> {
    match model_id {
        IPHONE_QWEN35_9B_ID => Some(IPHONE_QWEN35_9B),
        ANDROID_GEMMA4_E2B_LITERTLM_ID => Some(ANDROID_GEMMA4_E2B_LITERTLM),
        ANDROID_GEMMA4_E2B_SM8750_LITERTLM_ID => Some(ANDROID_GEMMA4_E2B_SM8750_LITERTLM),
        ANDROID_QWEN3_4B_ID => Some(ANDROID_QWEN3_4B),
        _ => None,
    }
}

pub const IPHONE_QWEN35_9B_ID: &str = IPHONE_QWEN35_9B.model_id;
pub const ANDROID_GEMMA4_E2B_LITERTLM_ID: &str = ANDROID_GEMMA4_E2B_LITERTLM.model_id;
pub const ANDROID_GEMMA4_E2B_SM8750_LITERTLM_ID: &str = ANDROID_GEMMA4_E2B_SM8750_LITERTLM.model_id;
pub const ANDROID_QWEN3_4B_ID: &str = ANDROID_QWEN3_4B.model_id;

/// A multi-file model artifact set for MLX-format models.
///
/// MLX models are Hugging Face repos (config + tokenizer + safetensors
/// shards), not single files, so pinning means pinning every file. Rust owns
/// the download of each file — resumable, revision-pinned, SHA-256-verified —
/// into one model directory that the platform bridge receives whole. `files`
/// is in download order: small config files first so a bad revision fails
/// before any multi-gigabyte shard transfer starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxArtifactSet {
    pub model_id: &'static str,
    pub repository: &'static str,
    pub revision: &'static str,
    pub files: &'static [ArtifactSpec],
    pub total_bytes: u64,
    /// Memory-preflight input: weights plus KV cache at the trusted context
    /// plus Metal buffer overhead. MLX's unified-memory residency runs well
    /// above the artifact bytes, unlike llama.cpp's mmap profile. Estimated
    /// until tuned against on-device jetsam evidence.
    pub estimated_peak_bytes: u64,
    /// How this model marks reasoning tokens, so the engine splits `<think>`
    /// blocks into `ThinkingDelta` instead of leaking them into content.
    pub reasoning_format: ReasoningFormat,
}

macro_rules! mlx_file {
    ($repo:literal, $rev:literal, $name:literal, $bytes:literal, $sha:literal) => {
        ArtifactSpec {
            repository: $repo,
            revision: $rev,
            file_name: $name,
            url: concat!(
                "https://huggingface.co/",
                $repo,
                "/resolve/",
                $rev,
                "/",
                $name
            ),
            byte_len: $bytes,
            sha256: $sha,
        }
    };
}

/// Qwen3-4B 4-bit MLX — the iOS default. Reasoning-tuned, fits the 8 GB device
/// tier. Text-only. Architecture is `qwen3`, which MLXLLM's Swift
/// `LLMModelFactory` registers (the pinned mlx-swift-examples 2.29.1 knows
/// `qwen2`/`qwen3`/`qwen3_moe` but NOT the newer `qwen3_5` — a Qwen3.5 MLX
/// build fails on device with `unsupportedModelType("qwen3_5")`). The
/// `model_id` keeps its historical `qwen3.5-4b-mlx-4bit` string only as a
/// stable identity across persisted policies; the artifact is Qwen3-4B.
pub const IOS_QWEN35_4B_MLX: MlxArtifactSet = MlxArtifactSet {
    model_id: "qwen3.5-4b-mlx-4bit",
    repository: "mlx-community/Qwen3-4B-4bit",
    revision: "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
    files: &[
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "config.json",
            937,
            "b5efdcf3b0035a3638e7228dad4d85f5c4a23f156eb7cdb0b44c8366a5d34d9b"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "tokenizer_config.json",
            9_706,
            "253153d0738ceb4c668d2eff957714dd2bea0b56de772a9fdccd96cbf517e6a0"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "added_tokens.json",
            707,
            "c0284b582e14987fbd3d5a2cb2bd139084371ed9acbae488829a1c900833c680"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "special_tokens_map.json",
            613,
            "76862e765266b85aa9459767e33cbaf13970f327a0e88d1c65846c2ddd3a1ecd"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "merges.txt",
            1_671_853,
            "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "vocab.json",
            2_776_833,
            "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "model.safetensors.index.json",
            63_924,
            "f7825defe5865d179c3b593173d37056be5f202dcb7153985cf74e75ecf1628b"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "tokenizer.json",
            11_422_654,
            "aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4"
        ),
        mlx_file!(
            "mlx-community/Qwen3-4B-4bit",
            "4dcb3d101c2a062e5c1d4bb173588c54ea6c4d25",
            "model.safetensors",
            2_263_022_529,
            "e240c0bdc0ebb0681bf0da0f98d9719fd6ebe269a3633f81542c13e81345651d"
        ),
    ],
    total_bytes: 2_278_969_756,
    estimated_peak_bytes: 3_400_000_000,
    reasoning_format: ReasoningFormat::Qwen3Think,
};

/// Qwen3-8B 4-bit MLX — the 12 GB-device opt-in tier. Architecture `qwen3`
/// (registered in mlx-swift-examples 2.29.1, unlike the newer `qwen3_5`). Never
/// a default: the memory preflight admits it only where the jetsam budget
/// clears `estimated_peak_bytes`. The `model_id` keeps its historical
/// `qwen3.5-9b-mlx-4bit` string as a stable identity; the artifact is Qwen3-8B.
pub const IOS_QWEN35_9B_MLX: MlxArtifactSet = MlxArtifactSet {
    model_id: "qwen3.5-9b-mlx-4bit",
    repository: "mlx-community/Qwen3-8B-4bit",
    revision: "545dc4251c05440727734bcd94334791f6ab0192",
    files: &[
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "config.json",
            939,
            "e5485285fd7e289e76e9cffa112f6dc2e3426519082f7db9b69041589f81a218"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "tokenizer_config.json",
            9_706,
            "253153d0738ceb4c668d2eff957714dd2bea0b56de772a9fdccd96cbf517e6a0"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "added_tokens.json",
            707,
            "c0284b582e14987fbd3d5a2cb2bd139084371ed9acbae488829a1c900833c680"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "special_tokens_map.json",
            613,
            "76862e765266b85aa9459767e33cbaf13970f327a0e88d1c65846c2ddd3a1ecd"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "merges.txt",
            1_671_853,
            "8831e4f1a044471340f7c0a83d7bd71306a5b867e95fd870f74d0c5308a904d5"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "vocab.json",
            2_776_833,
            "ca10d7e9fb3ed18575dd1e277a2579c16d108e32f27439684afa0e10b1440910"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "model.safetensors.index.json",
            64_065,
            "3fb25463b4078b1fc27159daa605190029c2e965f533bf0b1b594f96cbfceb8a"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "tokenizer.json",
            11_422_654,
            "aeb13307a71acd8fe81861d94ad54ab689df773318809eed3cbe794b4492dae4"
        ),
        mlx_file!(
            "mlx-community/Qwen3-8B-4bit",
            "545dc4251c05440727734bcd94334791f6ab0192",
            "model.safetensors",
            4_607_835_174,
            "f2d29621aab300336ad645567ff38c42aac755513006ef4e8a579cf7ef5256d8"
        ),
    ],
    total_bytes: 4_623_782_544,
    estimated_peak_bytes: 6_200_000_000,
    reasoning_format: ReasoningFormat::Qwen3Think,
};

pub const IOS_QWEN35_4B_MLX_ID: &str = IOS_QWEN35_4B_MLX.model_id;
pub const IOS_QWEN35_9B_MLX_ID: &str = IOS_QWEN35_9B_MLX.model_id;

pub fn mlx_artifact_set_for_id(model_id: &str) -> Option<&'static MlxArtifactSet> {
    match model_id {
        IOS_QWEN35_4B_MLX_ID => Some(&IOS_QWEN35_4B_MLX),
        IOS_QWEN35_9B_MLX_ID => Some(&IOS_QWEN35_9B_MLX),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifacts_are_revision_pinned_and_sized() {
        for set in [
            IPHONE_QWEN35_9B,
            ANDROID_GEMMA4_E2B_LITERTLM,
            ANDROID_GEMMA4_E2B_SM8750_LITERTLM,
            ANDROID_QWEN3_4B,
        ] {
            for artifact in std::iter::once(set.model).chain(set.projector) {
                assert!(!artifact.revision.is_empty());
                assert!(artifact.url.contains(artifact.revision));
                assert!(artifact.byte_len > 0);
                assert_eq!(artifact.sha256.len(), 64);
            }
        }
    }

    #[test]
    fn platform_defaults_are_typed_and_distinct() {
        assert_eq!(
            MobilePlatform::Ios.default_model().model_id,
            IPHONE_QWEN35_9B_ID
        );
        assert_eq!(
            MobilePlatform::Android.default_model().model_id,
            ANDROID_GEMMA4_E2B_LITERTLM_ID
        );
        assert_ne!(
            MobilePlatform::Ios.default_model().model_id,
            MobilePlatform::Android.default_model().model_id
        );
        assert_eq!(
            artifact_set_for_id(IPHONE_QWEN35_9B_ID),
            Some(IPHONE_QWEN35_9B)
        );
        assert_eq!(
            artifact_set_for_id(ANDROID_GEMMA4_E2B_LITERTLM_ID),
            Some(ANDROID_GEMMA4_E2B_LITERTLM)
        );
    }

    #[test]
    fn mlx_file_sets_are_revision_pinned_sized_and_summed() {
        for set in [&IOS_QWEN35_4B_MLX, &IOS_QWEN35_9B_MLX] {
            assert!(!set.revision.is_empty());
            assert!(!set.files.is_empty());
            let mut sum = 0u64;
            for file in set.files {
                assert_eq!(file.repository, set.repository);
                assert_eq!(file.revision, set.revision);
                assert!(file.url.contains(set.revision));
                assert!(file.url.ends_with(file.file_name));
                assert!(file.byte_len > 0);
                assert_eq!(file.sha256.len(), 64);
                sum += file.byte_len;
            }
            assert_eq!(
                sum, set.total_bytes,
                "{}: total_bytes must equal the sum of its files",
                set.model_id
            );
            assert!(
                set.estimated_peak_bytes > set.total_bytes,
                "{}: peak residency must exceed artifact bytes (KV cache + Metal overhead)",
                set.model_id
            );
        }
    }

    #[test]
    fn mlx_sets_order_configs_before_shards() {
        // Download order is the array order; a bad revision must fail on a
        // small config file, never after gigabytes of shard transfer.
        for set in [&IOS_QWEN35_4B_MLX, &IOS_QWEN35_9B_MLX] {
            let first_shard = set
                .files
                .iter()
                .position(|file| file.file_name.ends_with(".safetensors"))
                .expect("every MLX set has at least one shard");
            let last_config = set
                .files
                .iter()
                .rposition(|file| !file.file_name.ends_with(".safetensors"))
                .expect("every MLX set has config files");
            assert!(
                last_config < first_shard,
                "{}: all non-shard files must precede the first shard",
                set.model_id
            );
        }
    }

    #[test]
    fn mlx_ids_resolve_and_ios_default_id_is_the_mlx_4b() {
        assert_eq!(
            mlx_artifact_set_for_id(IOS_QWEN35_4B_MLX_ID),
            Some(&IOS_QWEN35_4B_MLX)
        );
        assert_eq!(
            mlx_artifact_set_for_id(IOS_QWEN35_9B_MLX_ID),
            Some(&IOS_QWEN35_9B_MLX)
        );
        assert_eq!(mlx_artifact_set_for_id("qwen3.5-9b-q4-k-m"), None);
        assert_eq!(MobilePlatform::Ios.default_model_id(), IOS_QWEN35_4B_MLX_ID);
        assert_eq!(
            MobilePlatform::Android.default_model_id(),
            ANDROID_GEMMA4_E2B_LITERTLM_ID
        );
    }
}
