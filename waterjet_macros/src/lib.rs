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
        
#[no_mangle]
pub extern "system" fn Java_{package}_{main_class}_{main_class}_1rust_1onEnable (
    env: JNIEnv,
    class: JClass,
    plugin: JObject,
) {{
    println!("this is rust saying hello men");
}}

#[no_mangle]
pub extern "system" fn Java_{package}_{main_class}_{main_class}_1rust_1onDisable (
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
