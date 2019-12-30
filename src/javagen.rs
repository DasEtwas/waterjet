const BASE_CLASS: &'static str = r#"package ${PACKAGE};

import org.bukkit.plugin.java.JavaPlugin;

public class ${NAME} extends JavaPlugin {

    private static boolean loadedLibs = false;

	static {
		try {
			System.load(new java.io.File(".").getCanonicalPath() + java.io.File.separator + "plugins" + java.io.File.separator + "lib" + java.io.File.separator + "${LIBNAME}");
			loadedLibs = true;
		} catch (UnsatisfiedLinkError e) {
            System.err.println("ERROR: Failed to load waterjet plugin library file ${LIBNAME}: " + e);
		} catch (java.io.IOException e) {
		    System.err.println("Failed to canonicalize path for plugin library file: " + e);
		}
	}

	static native void ${NAME}_rust_onEnable(JavaPlugin plugin);

	static native void ${NAME}_rust_onDisable(JavaPlugin plugin);

	private Thread rust_thread;

	@Override
	public void onEnable() {
	    if (loadedLibs) {
            ${NAME}_rust_onEnable((JavaPlugin) this);
        }
	}

	@Override
	public void onDisable() {
	    if (loadedLibs) {
		    ${NAME}_rust_onDisable((JavaPlugin) this);
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
