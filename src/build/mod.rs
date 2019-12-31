use case::CaseExt;
use javagen::source_files;
use joinery::JoinableIterator;
use std::collections::HashMap;
use std::fs::{read_dir, DirBuilder, DirEntry, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

mod javagen;

// needs to be manually synced with waterjet_macros::PACKAGE_PREFIX
pub const PACKAGE_PREFIX: &str = "io.github.waterjet";

#[cfg(windows)]
pub const LIB_FILE_SUFFIX: &str = "dll";
#[cfg(windows)]
pub const LIB_FILE_PREFIX: &str = "";
#[cfg(unix)]
pub const LIB_FILE_SUFFIX: &str = "so";
#[cfg(unix)]
pub const LIB_FILE_PREFIX: &str = "lib";

/// Should be called by the plugin crate's build script
pub fn build() {
    let (plugin_name, plugin_yaml) = make_plugin_yaml().expect("failed to read Cargo.toml");

    let source_files = source_files(
        format!(
            "{}{}.{}",
            LIB_FILE_PREFIX,
            std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME not set"),
            LIB_FILE_SUFFIX
        )
        .as_str(),
        plugin_name.as_str(),
        PACKAGE_PREFIX,
    );

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));

    let package_dirs = {
        let mut path = out_dir.to_owned();
        for dir in PACKAGE_PREFIX.split(".") {
            path = path.join(dir);
        }
        path
    };

    File::create(&out_dir.join("plugin.yml"))
        .expect("failed to create plugin.yml")
        .write_all(plugin_yaml.as_bytes())
        .expect("failed to write plugin.yml");

    // this could be a first time run, ignore No such file error
    let _ = std::fs::remove_dir_all(&package_dirs);

    DirBuilder::new()
        .recursive(true)
        .create(&package_dirs)
        .expect("failed to create plugin source code directories");

    // this could be a first time run, ignore No such file error
    let _ = std::fs::remove_dir_all(out_dir.join("compiled"));

    DirBuilder::new()
        .create(out_dir.join("compiled"))
        .expect("failed to create folder for compiled java files");

    let mut source_paths = vec![];
    for (name, content) in source_files {
        let path = package_dirs.join(format!("{}.java", name));
        source_paths.push(path.to_owned());

        File::create(&path)
            .expect("failed to create source code file")
            .write_all(content.as_bytes())
            .expect("failed to write java source");
    }

    {
        let javac_out = Command::new("javac")
            .current_dir(&out_dir)
            .args(
                [
                    "-Xlint:deprecation",
                    "-d",
                    "./compiled",
                    "-cp",
                    format!(
                        "{}/jars/*",
                        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set")
                    )
                    .as_str(),
                ]
                .iter()
                .cloned()
                .chain(source_paths.iter().map(|path| {
                    path.strip_prefix(&out_dir)
                        .expect("failed to strip file")
                        .to_str()
                        .expect("file path not valid UTF-8")
                }))
                .collect::<Vec<&str>>()
                .as_slice(),
            )
            .status()
            .expect("failed to run javac command");

        if javac_out.code().is_some() && javac_out.code().unwrap() != 0 {
            panic!("javac exited with {}", javac_out);
        } else {
            println!("javac exited with {}", javac_out);
        }
    }

    let class_files = find_files_recursively(out_dir.join("compiled"), ".class")
        .expect("failed to search for .class files");

    {
        let jar_out = Command::new("jar")
            .current_dir(&out_dir.join("compiled"))
            .args(
                ["-cvfM", "../plugin.jar"]
                    .iter()
                    .cloned()
                    .chain(class_files.iter().map(|s| {
                        s.strip_prefix(&out_dir.join("compiled"))
                            .expect("failed to strip file")
                            .to_str()
                            .expect("file path not valid UTF-8")
                    }))
                    .collect::<Vec<&str>>()
                    .as_slice(),
            )
            .status()
            .expect("failed to run javac command");

        if jar_out.code().is_some() && jar_out.code().unwrap() != 0 {
            panic!("jar exited with {}", jar_out);
        } else {
            println!("jar exited with {}", jar_out);
        }
    }

    {
        let jar_out = Command::new("jar")
            .current_dir(&out_dir)
            .args(&["-uvf", "plugin.jar", "plugin.yml"])
            .status()
            .expect("failed to run jar command");

        if jar_out.code().is_some() && jar_out.code().unwrap() != 0 {
            panic!("jar exited with {}", jar_out);
        } else {
            println!("jar exited with {}", jar_out);
        }
    }

    let jar_dest = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!("{}.jar", plugin_name));

    println!("copying plugin jar to {}", jar_dest.to_str().unwrap());
    std::fs::copy(out_dir.join("plugin.jar"), jar_dest)
        .expect("failed to copy plugin jar to target folder");
}

fn find_files_recursively<P: AsRef<Path>>(
    path: P,
    file_name_end: &str,
) -> std::io::Result<Vec<PathBuf>> {
    let mut matches = vec![];
    fn search_for_class(entry: DirEntry, matches: &mut Vec<PathBuf>, file_name_end: &str) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let file_name = entry
                    .file_name()
                    .to_str()
                    .expect("file name is not valid UTF-8")
                    .to_owned();

                if file_name.ends_with(file_name_end) {
                    matches.push(entry.path());
                }
            }

            if metadata.is_dir() {
                if let Ok(read_dir) = read_dir(entry.path()) {
                    for entry in read_dir {
                        if let Ok(entry) = entry {
                            search_for_class(entry, matches, file_name_end);
                        }
                    }
                }
            }
        }
    };

    for entry in read_dir(path)? {
        if let Ok(entry) = entry {
            search_for_class(entry, &mut matches, file_name_end);
        }
    }

    Ok(matches)
}

/// returns the plugin's main class name given by package.metadata.waterjet.name in Cargo.toml, and the rendered plugin.yml
pub fn make_plugin_yaml() -> Result<(String, String), std::io::Error> {
    // https://www.spigotmc.org/wiki/plugin-yml/
    let mut plugin_attributes = HashMap::new();

    const ENV_VAR_PARAMETERS: &[(&str, &str)] = &[
        ("CARGO_PKG_AUTHORS", "authors"),
        ("CARGO_PKG_DESCRIPTION", "description"),
        ("CARGO_PKG_HOMEPAGE", "website"),
        ("CARGO_PKG_VERSION", "version"),
        ("CARGO_PKG_NAME", "name"),
    ];

    for (env_var, param_name) in ENV_VAR_PARAMETERS {
        if let Ok(param) = std::env::var(env_var) {
            plugin_attributes.insert(
                (*param_name).to_owned(),
                StringOrList::String(match *param_name {
                    "name" => param.to_camel(),
                    _ => param,
                }),
            );
        }
    }

    plugin_attributes.insert(
        "api-version".to_owned(),
        StringOrList::String("1.13".to_owned()),
    );

    parse_plugin_attributes(&mut plugin_attributes)?;

    let mut plugin_yml = String::new();

    // required fields
    let name = plugin_attributes
        .get("name")
        .and_then(StringOrList::as_string) // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
        .expect("required plugin.yml attribute not given: name (see package.metadata.waterjet.name or set CARGO_PKG_NAME)");
    plugin_yml.push_str(format!("name: {}\n", name).as_str());

    let main = format!("{}.{}\n", PACKAGE_PREFIX, name);
    plugin_yml.push_str(format!("main: {}", main).as_str());

    let version = plugin_attributes
        .get("version")
        .and_then(StringOrList::as_string)
        .map(|s| s.lines().join_with(" ").to_string())
        .expect("required plugin.yml attribute not given: version (see package.metadata.waterjet.version or set CARGO_PKG_VERSION)");
    plugin_yml.push_str(format!("version: {}\n", version).as_str());

    // optional automatic fields
    if let Some(authors) = plugin_attributes
        .get("authors")
        .cloned()
        .map(StringOrList::into_list)
        .as_ref()
        .map(StringOrList::to_yaml_string)
    {
        plugin_yml.push_str(format!("authors: {}\n", authors).as_str());
    }

    if let Some(description) = plugin_attributes
        .get("description")
        .and_then(StringOrList::as_string)
    {
        plugin_yml.push_str(
            format!(
                "description: |\n{}",
                description
                    .lines()
                    .map(|line| format!("  {}\n", line))
                    .collect::<String>()
            )
            .as_str(),
        );
    }

    if let Some(website) = plugin_attributes
        .get("website")
        .and_then(StringOrList::as_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("website: {}\n", website).as_str());
    }

    if let Some(api_version) = plugin_attributes
        .get("api-version")
        .and_then(StringOrList::as_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("api-version: {}\n", api_version).as_str());
    }

    if let Some(prefix) = plugin_attributes
        .get("prefix")
        .and_then(StringOrList::as_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("prefix: {}\n", prefix).as_str());
    }

    if let Some(loadbefore) = plugin_attributes
        .get("loadbefore")
        .cloned()
        .map(StringOrList::into_list)
        .as_ref()
        .map(StringOrList::to_yaml_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("loadbefore: {}\n", loadbefore).as_str());
    }

    if let Some(depend) = plugin_attributes
        .get("depend")
        .cloned()
        .map(StringOrList::into_list)
        .as_ref()
        .map(StringOrList::to_yaml_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("depend: {}\n", depend).as_str());
    }

    if let Some(softdepend) = plugin_attributes
        .get("softdepend")
        .cloned()
        .map(StringOrList::into_list)
        .as_ref()
        .map(StringOrList::to_yaml_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("softdepend: {}\n", softdepend).as_str());
    }

    if let Some(extra) = plugin_attributes
        .get("extra")
        .and_then(StringOrList::as_string)
    {
        plugin_yml.push_str(&extra);
    }

    Ok((name, plugin_yml))
}

/// Used in waterjet's build.rs to generate a plugin.yml.
/// The automatically filled fields are only lists or strings.
#[derive(Clone)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

impl StringOrList {
    pub fn as_string(&self) -> Option<&String> {
        match self {
            StringOrList::String(s) => Some(&s),
            _ => None,
        }
    }

    /// Converts String into List
    pub fn into_list(self) -> StringOrList {
        match self {
            StringOrList::String(s) => StringOrList::List(vec![s]),
            x => x,
        }
    }

    pub fn to_yaml_string(&self) -> String {
        match self {
            StringOrList::String(s) => s.clone(),
            StringOrList::List(list) => format!(
                "[{}]",
                list.iter()
                    .map(|elem| format!("{}", elem))
                    .join_with(", ")
                    .to_string()
            ),
        }
    }
}

pub fn parse_plugin_attributes(props: &mut HashMap<String, StringOrList>) -> std::io::Result<()> {
    let cargo_toml_path =
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");

    let mut cargo_toml = String::new();

    File::open(cargo_toml_path)?.read_to_string(&mut cargo_toml)?;

    if let Ok(toml) = cargo_toml.parse::<toml::Value>() {
        if let Some(package) = toml.get("package") {
            if let Some(metadata) = package.get("metadata") {
                if let Some(waterjet) = metadata.get("waterjet") {
                    if let Some(pkg) = waterjet.as_table() {
                        for (k, v) in pkg {
                            if let Some(v) = v.as_str() {
                                props.insert(k.clone(), StringOrList::String(v.to_string()));
                            } else {
                                if let Some(list) = v.as_array() {
                                    let string_vec = list
                                        .iter()
                                        .map(|value| value.as_str())
                                        .collect::<Vec<Option<&str>>>();
                                    if string_vec.iter().any(|opt| opt.is_none()) {
                                        props.insert(
                                            k.clone(),
                                            StringOrList::List(
                                                string_vec
                                                    .into_iter()
                                                    .map(|opt| opt.unwrap().to_owned())
                                                    .collect::<Vec<String>>(),
                                            ),
                                        );
                                    } else {
                                        println!(
                                            "package.metadata.waterjet.{} an array with not only strings",
                                            k
                                        );
                                    }
                                } else {
                                    println!(
                                        "package.metadata.waterjet.{} is not a string or array",
                                        k
                                    );
                                }
                            }
                        }
                    } else {
                        println!("package.metadata.waterjet is not a table");
                    }
                } else {
                    println!("package.metadata.waterjet does not exist");
                }
            } else {
                println!("package.metadata does not exist");
            }
        } else {
            println!("package does not exist");
        }
    } else {
        println!("TOML parsing error");
    }

    Ok(())
}
