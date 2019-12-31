use jni::objects::GlobalRef;
use jni::JNIEnv;
use waterjet::McPlugin;

waterjet::hook!(PrintHello, Model);

struct Model {}

impl Default for Model {
    fn default() -> Self {
        Model {}
    }
}

impl McPlugin for Model {
    fn on_enable(&self, plugin: &GlobalRef, jni: &JNIEnv) {
        println!("Hello, World!");
    }

    fn on_disable(&self, plugin: &GlobalRef, jni: &JNIEnv) {
        println!("Bye, World!");
    }
}
