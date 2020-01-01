# waterjet ![GitHub](https://img.shields.io/github/license/DasEtwas/waterjet?style=flat-square) ![GitHub top language](https://img.shields.io/github/languages/top/DasEtwas/waterjet?style=flat-square)

A safe abstraction for making Minecraft Bukkit Server Plugins in Rust using JNI.
(early prototype)

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
- Start the server and you should see `Hello, World!` appear after the message indicating that `[NameOfYourPluginsMainClass] version x.y.z` was loaded

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

  **Note**: The `extra` key may be used for anything which should be appended to the generated plugin.yml

- waterjet automatically generates a `NameOfYourPluginsMainClass.jar` file with all the necessary FFI code (currently `onEnable` and `onDisable`) which is linked to by the JVM against functions generated in the `waterjet::hook` proc-macro
- `NameOfYourPluginsMainClass` must be consistent across the Cargo.toml's value `package.metadata.waterjet.name` and the first argument supplied to the `waterjet::hook` proc-macro

## Todo

- Event handling
- Currently, the API is not on solid ground and must be redesigned for event handling, which would include rethinking the current threaded approach of the onEnable FFI function