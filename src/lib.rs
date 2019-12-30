pub mod build;
pub mod javagen;
pub use waterjet_macros::hook;

pub mod jni_hook_prelude {
    pub use jni::objects::{JClass, JObject};
    pub use jni::JNIEnv;
}
