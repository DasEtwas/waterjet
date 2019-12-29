pub mod build;
pub mod hook;
pub use waterjet_macros::hook;

pub mod jni_hook_prelude {
    pub use jni::objects::{JClass, JObject};
    pub use jni::JNIEnv;
}
