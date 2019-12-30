const BASE_CLASS: &'static str = r#"package ${PACKAGE};

import org.bukkit.plugin.java.JavaPlugin;

public class ${NAME} extends JavaPlugin {

    private static boolean loadedLibs = false;

	static {
		try {
			System.loadLibrary("${LIBNAME}");
			loadedLibs = true;
		} catch (UnsatisfiedLinkError e) {
            System.out.println("ERROR: Failed to load waterjet Plugin library file ./${LIBNAME}");
            System.err.println("ERROR: Failed to load waterjet Plugin library file ./${LIBNAME}");
		}
	}

	static native void ${NAME}_rust_onEnable(JavaPlugin plugin);

	static native void ${NAME}_rust_onDisable(JavaPlugin plugin);

	private Thread rust_thread;

	@Override
	public void onEnable() {
	    if (loadedLibs) {
	        JavaPlugin plugin = (JavaPlugin) this;
	        
		    this.rust_thread = new Thread(new Runnable() {
                public void run() {
                    ${NAME}_rust_onEnable(plugin);
                }
            });
            this.rust_thread.start();
        }
	}

	@Override
	public void onDisable() {
	    if (loadedLibs) {
		    ${NAME}_rust_onDisable(this);
		}
	}
}
"#;

pub(crate) fn main_class(lib_file_name: &str, main_class: &str, package: &str) -> String {
    BASE_CLASS
        .replace("${LIBNAME}", lib_file_name)
        .replace("${NAME}", main_class)
        .replace("${PACKAGE}", package)
}
