extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};

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
pub extern "system" fn Java_waterjet_com_github_{main_class} (
    env: JNIEnv,
    class: JClass,
    plugin: JObject,
) {{
}}"#,
        main_class = main_class
    )
    .parse()
    .unwrap()
}
