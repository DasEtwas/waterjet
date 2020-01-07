use waterjet::{
    jni::{objects::GlobalRef, JNIEnv},
    McPlugin,
};

waterjet::hook!(PrintHello, Model);

struct Model {}

impl Default for Model {
    fn default() -> Self {
        Model {}
    }
}

impl McPlugin for Model {
    fn on_enable(&self, _plugin: &GlobalRef, _jni: &JNIEnv) {
        println!("Hello, World!");
    }

    fn on_disable(&self, _plugin: &GlobalRef, _jni: &JNIEnv) {
        println!("Bye, World!");
    }
}
