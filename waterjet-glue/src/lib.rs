use joinery::JoinableIterator;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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
