# waterjet ![GitHub](https://img.shields.io/github/license/DasEtwas/waterjet?style=flat-square) ![GitHub top language](https://img.shields.io/github/languages/top/DasEtwas/waterjet?style=flat-square)

A plugin generator and safe abstraction for making Minecraft Bukkit Server Plugins in Rust using JNI.

**(early prototype)**

## Quick start guide
- Create a new Cargo library project (`cargo new --lib`)
- Add waterjet as dependency and build dependency
- Edit the project's Cargo.toml to include to following lines:

```toml
[package]
# ...
build = "build.rs"

[lib]
crate_type = ["cdylib"]

[package.metadata.waterjet]
name = "NameOfYourPluginsMainClass"
```
- Create a build script (`build.rs` in this example) which includes the following lines:

```rust
fn main() {
    waterjet::build::build();
}
```
- Add these lines to your `src/lib.rs`

```rust
use waterjet::{
    jni::{objects::GlobalRef, JNIEnv},
    McPlugin,
};

waterjet::hook!(NameOfYourPluginsMainClass, Model);

struct Model {}

impl Default for Model {
    fn default() -> Self {
        Model {}
    }
}

impl McPlugin for Model {
    fn on_enable(&self, _plugin: &GlobalRef, jni: &JNIEnv) {
        println!("Hello, World!");
    }

    fn on_disable(&self, _plugin: &GlobalRef, _jni: &JNIEnv) {
        println!("Bye, World!");
    }
}
```
- Build the project using `cargo build`
- Copy the generated `.jar` file from the `target/$PROFILE` folder into your server's `plugins` directory and the `.so`/`.dll` file into `plugins/lib`
- Start the server and you should see `Hello, World!` appear after the message indicating that `[NameOfYourPluginsMainClass] version x.y.z` was enabled

## Examining the above example

- The function `waterjet::build::build` parses your packages Cargo.toml for values under `package.metadata.waterjet` and converts them to a `plugin.yml` which is included in the generated JAR file. **Supported keys include**:

  Key | Default value | Hint
  ---|---|---
  `name`|camel-cased `CARGO_PKG_NAME`|single-line string
  `authors`|(`CARGO_PKG_AUTHORS`)|list or string
  `website`|(`CARGO_PKG_HOMEPAGE`)|single-line string
  `description`|(`CARGO_PKG_DESCRIPTION`)|optionally multi-line string
  `version`|(`CARGO_PKG_VERSION`)|single-line string
  `api-version`|1.13|single-line string
  `prefix`||single-line string
  `loadbefore`||list or string
  `depend`||list or string
  `softdepend`||list or string
  `extra`||multi-line string containing YAML

  **Note**: The `extra` key may be used for anything which should be appended to the generated plugin.yml (eg. permissions, commands)
  **Note**: For info on the keys supported by Spigot: https://www.spigotmc.org/wiki/plugin-yml/

- waterjet automatically generates a `NameOfYourPluginsMainClass.jar` file with all the necessary FFI code (currently `onEnable` and `onDisable`) which is linked to by the JVM against functions generated in the `waterjet::hook` proc-macro
- `NameOfYourPluginsMainClass` must be consistent across the Cargo.toml's value `package.metadata.waterjet.name` and the first argument supplied to the `waterjet::hook` proc-macro

## Features

- Generation of JAR containing plugin code which calls Rust code
- Automatic loading of Rust code from the plugin
- **Compatibility with `/reload` command**: Rust code is loaded by a custom classloader which is destructed in `onDisable` (see todo)
- Trait providing basic plugin methods (`onEnable`, `onDisable`) providing a reference to the `JavaPlugin` instance and to the `JNIEnv`

## Todo

- Event handling
- Currently, the API is not on solid ground and must be redesigned for event handling, which would include rethinking the current threaded approach of the onEnable FFI function
- `onDisable` uses a `System.gc()` call to free the `ClassLoader` which loaded the plugin's Rust code, which may cause performance problems (quick and dirty solution to make `/reload` work)
