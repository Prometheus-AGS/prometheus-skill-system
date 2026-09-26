//! Android JNI binding for the official LiteRT-LM Android API.
//!
//! Rust still owns model selection, context construction, tool policy, and
//! event normalization. This module only bridges into the app-local
//! `__APP_ID__.__APP_CLASS__LiteRtLmBridge` adapter, which binds Google's
//! Kotlin/Android LiteRT-LM package.

use std::path::Path;
use std::sync::OnceLock;

use gen_ui_types::error::{CoreError, CoreResult};
use gen_ui_types::inference::{InferenceRequest, InferenceToolResult};
use jni::objects::{Global, JClass, JObject, JString, JValue};
use jni::signature::RuntimeMethodSignature;
use jni::strings::JNIString;
use jni::{jni_sig, jni_str, Env, JavaVM};
use serde::Deserialize;

static JAVA_VM: OnceLock<JavaVM> = OnceLock::new();
static BRIDGE_CLASS: OnceLock<Global<JClass<'static>>> = OnceLock::new();

pub fn set_java_vm(vm: JavaVM) {
    let _ = JAVA_VM.set(vm);
}

pub fn java_vm_available() -> bool {
    JAVA_VM.get().is_some()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeLiteRtLmResponse {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub tool_calls: Vec<NativeLiteRtLmToolCall>,
    #[serde(default)]
    pub tool_result_fallback_applied: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NativeLiteRtLmToolCall {
    pub name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

pub struct NativeLiteRtLmAdapter;

impl NativeLiteRtLmAdapter {
    pub fn load(model_path: &Path, cache_dir: &Path) -> CoreResult<Self> {
        with_env("LiteRT-LM JavaVM", |env| {
            let model_path = env
                .new_string(model_path.to_string_lossy().as_ref())
                .map_err(|error| jni_error(env, "LiteRT-LM model path string", error))?;
            let cache_dir = env
                .new_string(cache_dir.to_string_lossy().as_ref())
                .map_err(|error| jni_error(env, "LiteRT-LM cache dir string", error))?;
            call_static_string(
                env,
                "load",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
                &[
                    JValue::Object(model_path.as_ref()),
                    JValue::Object(cache_dir.as_ref()),
                ],
            )?;
            Ok(())
        })?;
        Ok(Self)
    }

    pub fn generate_text(&self, request: &InferenceRequest) -> CoreResult<NativeLiteRtLmResponse> {
        let request_json = serde_json::to_string(request)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM request JSON: {error}")))?;
        let response = with_env("LiteRT-LM JavaVM", |env| {
            let request_json = env
                .new_string(request_json)
                .map_err(|error| jni_error(env, "LiteRT-LM request string", error))?;
            call_static_string(
                env,
                "sendMessage",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(request_json.as_ref())],
            )
        })?;
        serde_json::from_str(&response)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM response JSON: {error}")))
    }

    pub fn continue_with_tool_results(
        &self,
        results: &[InferenceToolResult],
    ) -> CoreResult<NativeLiteRtLmResponse> {
        let results_json = serde_json::to_string(results)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM tool results JSON: {error}")))?;
        let response = with_env("LiteRT-LM JavaVM", |env| {
            let results_json = env
                .new_string(results_json)
                .map_err(|error| jni_error(env, "LiteRT-LM tool results string", error))?;
            call_static_string(
                env,
                "sendToolResults",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(results_json.as_ref())],
            )
        })?;
        serde_json::from_str(&response)
            .map_err(|error| CoreError::Serde(format!("LiteRT-LM response JSON: {error}")))
    }

    pub fn cancel() {
        let _ = with_env("LiteRT-LM JavaVM", |env| {
            call_static_void(env, "cancel", "()V", &[])
        });
    }

    pub fn close() {
        let _ = with_env("LiteRT-LM JavaVM", |env| {
            call_static_void(env, "close", "()V", &[])
        });
    }
}

#[derive(Debug)]
enum AttachError {
    Jni(jni::errors::Error),
    Core(CoreError),
}

impl From<jni::errors::Error> for AttachError {
    fn from(error: jni::errors::Error) -> Self {
        Self::Jni(error)
    }
}

fn with_env<T, F>(context: &str, operation: F) -> CoreResult<T>
where
    F: for<'local> FnOnce(&mut Env<'local>) -> CoreResult<T>,
{
    let vm = JAVA_VM
        .get()
        .ok_or_else(|| CoreError::Terminal(format!("{context}: JNI_OnLoad has not run")))?;
    match vm.attach_current_thread(|env| operation(env).map_err(AttachError::Core)) {
        Ok(value) => Ok(value),
        Err(AttachError::Core(error)) => Err(error),
        Err(AttachError::Jni(error)) => Err(CoreError::Terminal(format!("{context}: {error}"))),
    }
}

fn call_static_string(
    env: &mut Env<'_>,
    method: &str,
    signature: &str,
    args: &[JValue<'_>],
) -> CoreResult<String> {
    let bridge_class = resolve_bridge_class(env)?;
    let method_name = method;
    let method = JNIString::new(method);
    let signature = RuntimeMethodSignature::from_str(signature)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge signature", error))?;
    let value = env
        .call_static_method(bridge_class, &method, signature.method_signature(), args)
        .map_err(|error| jni_error(env, &format!("LiteRT-LM bridge {method_name}"), error))?;
    let object = value
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM bridge result", error))?;
    let text = JString::cast_local(env, object)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge result type", error))?;
    text.try_to_string(env)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge result string", error))
}

fn call_static_void(
    env: &mut Env<'_>,
    method: &str,
    signature: &str,
    args: &[JValue<'_>],
) -> CoreResult<()> {
    let bridge_class = resolve_bridge_class(env)?;
    let method = JNIString::new(method);
    let signature = RuntimeMethodSignature::from_str(signature)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge signature", error))?;
    env.call_static_method(bridge_class, &method, signature.method_signature(), args)
        .map(|_| ())
        .map_err(|error| jni_error(env, "LiteRT-LM bridge call", error))
}

fn resolve_bridge_class(env: &mut Env<'_>) -> CoreResult<&'static Global<JClass<'static>>> {
    if let Some(global) = BRIDGE_CLASS.get() {
        return Ok(global);
    }

    let activity_thread = env
        .find_class(jni_str!("android/app/ActivityThread"))
        .map_err(|error| jni_error(env, "LiteRT-LM ActivityThread class", error))?;
    let application = env
        .call_static_method(
            activity_thread,
            jni_str!("currentApplication"),
            jni_sig!("()Landroid/app/Application;"),
            &[],
        )
        .map_err(|error| jni_error(env, "LiteRT-LM current application", error))?
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM current application object", error))?;
    if application.is_null() {
        return Err(CoreError::Terminal(
            "LiteRT-LM current application is not available".into(),
        ));
    }
    let class_loader = env
        .call_method(
            &application,
            jni_str!("getClassLoader"),
            jni_sig!("()Ljava/lang/ClassLoader;"),
            &[],
        )
        .map_err(|error| jni_error(env, "LiteRT-LM application class loader", error))?
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM application class loader object", error))?;
    let class_name = env
        .new_string("__APP_ID__.__APP_CLASS__LiteRtLmBridge")
        .map_err(|error| jni_error(env, "LiteRT-LM bridge class name", error))?;
    let class_name = JObject::from(class_name);
    let class = env
        .call_method(
            &class_loader,
            jni_str!("loadClass"),
            jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
            &[JValue::Object(class_name.as_ref())],
        )
        .map_err(|error| jni_error(env, "LiteRT-LM bridge loadClass", error))?
        .l()
        .map_err(|error| jni_error(env, "LiteRT-LM bridge class object", error))?;
    let class = JClass::cast_local(env, class)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge class type", error))?;
    let global = env
        .new_global_ref(class)
        .map_err(|error| jni_error(env, "LiteRT-LM bridge global class", error))?;
    let _ = BRIDGE_CLASS.set(global);
    BRIDGE_CLASS
        .get()
        .ok_or_else(|| CoreError::Terminal("LiteRT-LM bridge class cache failed".into()))
}

fn jni_error(env: &mut Env<'_>, context: &str, error: jni::errors::Error) -> CoreError {
    if env.exception_check() {
        env.exception_describe();
        env.exception_clear();
    }
    CoreError::Terminal(format!("{context}: {error}"))
}
