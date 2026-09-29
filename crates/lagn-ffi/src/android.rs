//! The Android side: the same operations, reachable from Kotlin without a C
//! shim. Built only with `--features jni`; an iOS build links none of it.
//!
//! Kotlin side:
//!
//! ```kotlin
//! object LagnNative {
//!     init { System.loadLibrary("lagn_ffi") }
//!     external fun init(configJson: String): String
//!     external fun topic(name: String, requestJson: String): String
//!     // ...
//! }
//! ```
//!
//! Strings are converted at the boundary, so Kotlin never sees a pointer and
//! nothing has to be freed by hand.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jstring};
use jni::JNIEnv;

use crate::{api_error_message, state};
use lagn_rules::resolve::Mode;
use lagn_server::api;

/// Hand a JSON string back to the JVM. If even that fails, return a null
/// pointer, which Kotlin sees as null and reports rather than crashing on.
fn out(env: &mut JNIEnv, s: String) -> jstring {
    match env.new_string(s) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn error(env: &mut JNIEnv, message: &str) -> jstring {
    out(env, serde_json::json!({ "error": message }).to_string())
}

fn arg(env: &mut JNIEnv, s: &JString) -> Option<String> {
    env.get_string(s).ok().map(|v| v.into())
}

/// Serialise a result the way the C ABI does, so both platforms agree.
fn result<T: serde::Serialize>(env: &mut JNIEnv, v: Result<T, lagn_server::ApiError>) -> jstring {
    match v {
        Ok(v) => match serde_json::to_string(&v) {
            Ok(s) => out(env, s),
            Err(e) => error(env, &format!("could not serialise the result: {e}")),
        },
        Err(e) => error(env, &api_error_message(&e)),
    }
}

macro_rules! jni_endpoint {
    ($name:ident, $req:ty, |$state:ident, $r:ident| $body:expr) => {
        #[no_mangle]
        pub extern "system" fn $name(mut env: JNIEnv, _class: JClass, request_json: JString) -> jstring {
            let Some(text) = arg(&mut env, &request_json) else {
                return error(&mut env, "a JSON request body is required");
            };
            let guard = match state().read() {
                Ok(g) => g,
                Err(_) => return error(&mut env, "the engine is in a failed state; restart the app"),
            };
            let Some($state) = guard.as_ref() else {
                return error(&mut env, "the engine is not initialised; call init first");
            };
            let $r: $req = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(e) => return error(&mut env, &format!("invalid request: {e}")),
            };
            result(&mut env, $body)
        }
    };
}

#[no_mangle]
pub extern "system" fn Java_app_lagn_LagnNative_init(mut env: JNIEnv, _class: JClass, config_json: JString) -> jstring {
    let Some(text) = arg(&mut env, &config_json) else {
        return error(&mut env, "init needs a JSON configuration string");
    };
    let c = match std::ffi::CString::new(text) {
        Ok(c) => c,
        Err(_) => return error(&mut env, "the configuration contained a NUL byte"),
    };
    // One implementation: the C entry point does the work.
    let p = unsafe { crate::lagn_init(c.as_ptr()) };
    if p.is_null() {
        return error(&mut env, "out of memory");
    }
    let s = unsafe { std::ffi::CStr::from_ptr(p) }.to_string_lossy().into_owned();
    unsafe { crate::lagn_string_free(p) };
    out(&mut env, s)
}

#[no_mangle]
pub extern "system" fn Java_app_lagn_LagnNative_ready(_env: JNIEnv, _class: JClass) -> jboolean {
    u8::from(crate::lagn_ready())
}

#[no_mangle]
pub extern "system" fn Java_app_lagn_LagnNative_version(mut env: JNIEnv, _class: JClass) -> jstring {
    let guard = match state().read() {
        Ok(g) => g,
        Err(_) => return error(&mut env, "the engine is in a failed state; restart the app"),
    };
    match guard.as_ref() {
        Some(s) => result(&mut env, Ok(api::version(s))),
        None => error(&mut env, "the engine is not initialised; call init first"),
    }
}

#[no_mangle]
pub extern "system" fn Java_app_lagn_LagnNative_topics(mut env: JNIEnv, _class: JClass) -> jstring {
    let guard = match state().read() {
        Ok(g) => g,
        Err(_) => return error(&mut env, "the engine is in a failed state; restart the app"),
    };
    match guard.as_ref() {
        Some(s) => result(&mut env, Ok(api::topics(s))),
        None => error(&mut env, "the engine is not initialised; call init first"),
    }
}

#[no_mangle]
pub extern "system" fn Java_app_lagn_LagnNative_topic(
    mut env: JNIEnv,
    _class: JClass,
    name: JString,
    request_json: JString,
) -> jstring {
    let (Some(name), Some(text)) = (arg(&mut env, &name), arg(&mut env, &request_json)) else {
        return error(&mut env, "a topic name and a JSON request body are required");
    };
    let guard = match state().read() {
        Ok(g) => g,
        Err(_) => return error(&mut env, "the engine is in a failed state; restart the app"),
    };
    let Some(s) = guard.as_ref() else {
        return error(&mut env, "the engine is not initialised; call init first");
    };
    let req: api::TopicRequest = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => return error(&mut env, &format!("invalid request: {e}")),
    };
    result(&mut env, api::topic(s, &name, req, Mode::Production))
}

jni_endpoint!(Java_app_lagn_LagnNative_chart, api::ChartRequest, |_s, req| api::chart(req));
jni_endpoint!(Java_app_lagn_LagnNative_periods, api::PeriodsRequest, |s, req| api::periods(s, req, Mode::Production));
jni_endpoint!(Java_app_lagn_LagnNative_family, api::FamilyRequest, |s, req| api::family(s, req, Mode::Production));
jni_endpoint!(Java_app_lagn_LagnNative_match_, api::MatchRequest, |s, req| api::match_(s, req, Mode::Production));
jni_endpoint!(Java_app_lagn_LagnNative_places, api::PlacesQuery, |s, q| api::places(s, &q));
jni_endpoint!(Java_app_lagn_LagnNative_offset, api::OffsetQuery, |s, q| api::offset(s, &q));
