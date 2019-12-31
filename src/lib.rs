pub mod build;
use jni::objects::GlobalRef;
use jni::{AttachGuard, JNIEnv};
pub use waterjet_macros::hook;

pub mod jni_hook_prelude {
    pub use jni::objects::{JClass, JObject};
    pub use jni::JNIEnv;
}

/// This trait is designed to be implemented by a plugin.
pub trait McPlugin: Default {
    /// Called when the Server calls Plugin.onEnable. Implementations of on_enable block execution of the Java method.
    fn on_enable(&self, plugin: &GlobalRef, jni: &JNIEnv) {}
    /// Called when the Server calls Plugin.onDisable. Implementations of on_disable block execution of the Java method.
    fn on_disable(&self, plugin: &GlobalRef, jni: &JNIEnv) {}
}

/// Represents a JNIEnv-owning Rust thread decoupled from the rest of the server.
/// Methods of this struct may only be called from the thread to which jni_env is attached. (Created in waterjet::hook!)
pub struct Model<'a, P: McPlugin> {
    plugin: P,
    java_plugin_ref: GlobalRef,
    jni_env: AttachGuard<'a>,
}

impl<'a, P> Model<'a, P>
where
    P: McPlugin,
{
    pub fn new(plugin: P, java_plugin_ref: GlobalRef, jni_env: AttachGuard<'a>) -> Model<P> {
        Model {
            plugin,
            java_plugin_ref,
            jni_env,
        }
    }

    pub fn on_enable(&self) {
        self.plugin.on_enable(&self.java_plugin_ref, &*self.jni_env);
    }

    pub fn on_disable(&self) {
        self.plugin
            .on_disable(&self.java_plugin_ref, &*self.jni_env);
    }
}
