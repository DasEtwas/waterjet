package ${PACKAGE};

import org.bukkit.plugin.java.JavaPlugin;

public final class ${NAME} extends JavaPlugin {
	
	static {
		try {
			System.loadLibrary("${LIBNAME}");
		} catch (UnsatisfiedLinkError e) {
		
		}
	}
	
	static native void ${NAME}_rust_onEnable(JavaPlugin plugin);
	
	static native void ${NAME}_rust_onDisable(JavaPlugin plugin);
	
	@Override
	public void onEnable() {
		${NAME}_rust_onEnable(this);
	}
	
	@Override
	public void onDisable() {
		${NAME}_rust_onDisable(this);
	}
}
