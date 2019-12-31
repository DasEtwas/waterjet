extern crate proc_macro;
use joinery::prelude::*;
use proc_macro::{TokenStream, TokenTree};

// needs to be manually synced with waterjet::build::PACKAGE_PREFIX
const PACKAGE_PREFIX: &str = "io.github.waterjet";

#[proc_macro]
pub fn hook(item: TokenStream) -> TokenStream {
    let main_class = match item
        .into_iter()
        .next()
        .expect("expected plugin main class name Group")
    {
        TokenTree::Ident(ident) => ident.to_string(),
        _ => panic!("plugin main class name must be a literal string (preferably CamelCase)"),
    };

    format!(
        r#"
use ::waterjet::jni_hook_prelude::*;
use std::{{sync::mpsc, thread, time::Duration}};
        
#[no_mangle]
pub extern "system" fn Java_{package}_{main_class}Native_rust_1{main_class}_1onEnable(
    env: JNIEnv,
    class: JClass,
    plugin: JObject,
) {{
    let jvm = env.get_java_vm().unwrap();

    let plugin = env.new_global_ref(plugin).unwrap();

    // used to block this function until thread started
    let (tx, rx) = mpsc::channel();

    let _ = thread::spawn(move || {{
        // Use the `JavaVM` interface to attach a `JNIEnv` to the current thread.
        let env = jvm.attach_current_thread().unwrap();
        
        // Signal that the thread has started.
        tx.send(()).unwrap();

        for i in 0..10 {{
            thread::sleep(Duration::from_millis(500));
            println!("hello men xd {{}}", i);
        }}
        
        // The current thread is detached automatically when `env` goes out of scope.
    }});

    // Wait until the thread has started.
    rx.recv().unwrap();
}}

#[no_mangle]
pub extern "system" fn Java_{package}_{main_class}Native_rust_1{main_class}_1onDisable(
    env: JNIEnv,
    class: JClass,
    plugin: JObject,
) {{
    println!("this is rust saying bye men");
}}"#,
        package = PACKAGE_PREFIX.split(".").join_with("_").to_string(),
        main_class = main_class
    )
    .parse()
    .unwrap()
}
