extern crate proc_macro;
use joinery::prelude::*;
use proc_macro::{TokenStream, TokenTree};

// needs to be manually synced with waterjet::build::PACKAGE_PREFIX
const PACKAGE_PREFIX: &str = "io.github.waterjet";

#[proc_macro]
pub fn hook(item: TokenStream) -> TokenStream {
    let mut tokens = item
        .into_iter()
        .map(|tt| match tt {
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None,
        })
        .collect::<Vec<Option<String>>>();

    let (main_class, model_type) = match tokens.as_slice() {
        // the None in the middle is Punct (,)
        [Some(_), None, Some(_)] => (tokens.remove(0).unwrap(), tokens.remove(1).unwrap()),
        _ => panic!("expected plugin main class name and model type hook!(MainClass, ModelType)"),
    };

    format!(
        r#"mod __java_hooks {{
    use super::{model_type};
    use ::waterjet::Model as __InternalModel;
    use ::waterjet::jni_hook_prelude::*;
    use ::std::{{sync::Arc, sync::atomic::AtomicBool, sync::atomic::Ordering, sync::mpsc, thread, time::Duration}};
    
    static mut MODEL: Option<Arc<__InternalModel<{model_type}>>> = None;
    static mut DISABLED: AtomicBool = AtomicBool::new(false);
    static mut THREAD: Option<std::thread::JoinHandle<()>> = None;
            
    #[no_mangle]
    pub extern "system" fn Java_{package}_{main_class}Native_rust_1{main_class}_1onEnable(
        env: JNIEnv,
        class: JClass,
        plugin: JObject,
    ) {{
        let jvm = env.get_java_vm().unwrap();
    
        let plugin = env.new_global_ref(plugin).unwrap();
    
        // used to block this function until thread is started and jvm attached to it
        let (tx, rx) = mpsc::channel();
        
        unsafe {{
            DISABLED.store(false, Ordering::Relaxed);
            MODEL = None;
            THREAD = None;
        }}
    
        unsafe {{
            THREAD = Some(thread::spawn(move || {{
               let env = jvm.attach_current_thread().unwrap();
               
               let mut model = __InternalModel::new({model_type}::default(), plugin, env);
               tx.send(()).unwrap();
               
               unsafe {{
                   MODEL = Some(Arc::new(model));
                   MODEL.as_ref().unwrap().on_enable();
               }}
               
               while !unsafe {{ DISABLED.load(Ordering::Relaxed) }} {{
                   thread::park();
               }}
               
               unsafe {{
                   MODEL.as_ref().unwrap().on_disable();
               }}
            }}));
        }}
    
        // Wait until the thread has started.
        rx.recv().unwrap();
    }}
    
    #[no_mangle]
    pub extern "system" fn Java_{package}_{main_class}Native_rust_1{main_class}_1onDisable(
        _: JNIEnv,
        _: JClass,
        _: JObject,
    ) {{
        unsafe {{
            DISABLED.store(true, Ordering::Relaxed);
            THREAD.as_ref().unwrap().thread().unpark();
        }}
    }}
}}"#,
        package = PACKAGE_PREFIX.split(".").join_with("_").to_string(),
        main_class = main_class,
        model_type = model_type
    )
    .parse()
    .unwrap()
}
