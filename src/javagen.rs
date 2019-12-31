const FILES: &[&str] = &[
    r#"package ${PACKAGE};

import org.bukkit.plugin.java.JavaPlugin;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.File;
import java.io.BufferedInputStream;
import java.lang.reflect.Method;

public class ${NAME} extends JavaPlugin {

    private CustomNativeLoader loader = null;
	
	private static ${NAME} instance;
	
	private static Class<?> native_;

	@Override
	public void onLoad() {
		instance = this;
		loader = new CustomNativeLoader();
        Class<?> c = loader.findClass(null);
        try {
        	Method m = c.getMethod("loadLibs");
        	m.setAccessible(true);
        	m.invoke(null);
        	m = null;
        	
        	native_ = c;
        } catch(Exception e) {
        	System.out.println("Failed to instantiate instance of native libs loader");
        	e.printStackTrace();
        }
	}

	@Override
	public void onEnable() {
	    try {
            Method m = native_.getMethod("rust_${NAME}_onEnable", JavaPlugin.class);
            m.setAccessible(true);
            m.invoke(null, this);
            m = null;
        } catch(Exception e) {
            System.err.println("Failed to call ${NAME} ffi: " + e);
            e.printStackTrace();
        }
	}

	@Override
	public void onDisable() {
		try {
            Method m = native_.getMethod("rust_${NAME}_onDisable", JavaPlugin.class);
            m.setAccessible(true);
            m.invoke(null, this);
            m = null;
        } catch(Exception e) {
            System.err.println("Failed to call ${NAME} ffi: " + e);
            e.printStackTrace();
        }
       
		loader = null;
		native_ = null;
		 
		// try to garbage collect the class loader to unload library
	    System.gc();
	}
	
	public class CustomNativeLoader extends ClassLoader {
        @Override
        public Class<?> findClass(String unused) {
            byte[] bytes = loadClassData("${PACKAGE}.${NAME}Native".replace(".", File.separator) + ".class");

            System.out.println("Loaded ${NAME} plugin ffi");
            return defineClass("${PACKAGE}.${NAME}Native", bytes, 0, bytes.length);
        }
    
        private byte[] loadClassData(String name) {
            InputStream is = instance.getResource(name);
            ByteArrayOutputStream baos = new ByteArrayOutputStream();
            
            try {
            	int i;
       			while ((i = is.read()) != -1) {
       			    baos.write(i);
       			}
       		
       			is.close();
       			byte[] classData = baos.toByteArray();
       			baos.close();
       			
            	return classData;
            } catch(IOException e) {
       			System.out.println("Failed to read class: " + e);
       			e.printStackTrace();
       			return null;
       		}
        }
    }
}
"#,
    r#"package ${PACKAGE};
    
import java.io.File;
import org.bukkit.plugin.java.JavaPlugin;

class ${NAME}Native {
	public static native void rust_${NAME}_onEnable(JavaPlugin plugin);

	public static native void rust_${NAME}_onDisable(JavaPlugin plugin);
	
    public static void loadLibs() {
        try {
            System.load(new File(".").getCanonicalPath() + File.separator + "plugins" + File.separator + "lib" + File.separator + "${LIBNAME}");
            System.out.println("Successfully loaded plugin library \"${LIBNAME}\"");
        } catch (UnsatisfiedLinkError e) {
            System.err.println("Failed to load waterjet plugin library file ${LIBNAME}: " + e);
        } catch (java.io.IOException e) {
            System.err.println("Failed to canonicalize path for plugin library file: " + e);
        }
    }
}"#,
];

/// Returns a vec of (file name, content)
pub(crate) fn source_files(
    lib_file_name: &str,
    main_class: &str,
    package: &str,
) -> Vec<(String, String)> {
    let file_names = [main_class.to_owned(), format!("{}Native", main_class)];

    FILES
        .to_vec()
        .into_iter()
        .zip(file_names.iter())
        .map(|(text, name)| {
            (
                name.to_owned(),
                text.replace("${LIBNAME}", lib_file_name)
                    .replace("${NAME}", main_class)
                    .replace("${PACKAGE}", package),
            )
        })
        .collect()
}
