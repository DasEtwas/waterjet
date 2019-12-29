use case::CaseExt;
use joinery::JoinableIterator;
use std::collections::HashMap;
use waterjet_glue::StringOrList;

// TODO error handling
pub fn build() {
    let (plugin_main_class_name, plugin_yaml) = make_plugin_yaml().unwrap();
}

pub fn make_plugin_yaml() -> Result<(String, String), std::io::Error> {
    // https://www.spigotmc.org/wiki/plugin-yml/
    let mut plugin_parameters = HashMap::new();

    const ENV_VAR_PARAMETERS: &[(&str, &str)] = &[
        ("CARGO_PKG_AUTHORS", "authors"),
        ("CARGO_PKG_DESCRIPTION", "description"),
        ("CARGO_PKG_HOMEPAGE", "website"),
        ("CARGO_PKG_VERSION", "version"),
        ("CARGO_PKG_NAME", "name"),
    ];

    for (env_var, param_name) in ENV_VAR_PARAMETERS {
        if let Ok(param) = std::env::var(env_var) {
            plugin_parameters.insert(
                (*param_name).to_owned(),
                StringOrList::String(match *param_name {
                    "name" => param.to_camel(),
                    _ => param,
                }),
            );
        }
    }

    waterjet_glue::parse_plugin_attributes(&mut plugin_parameters)?;

    let mut plugin_yml = String::new();

    // required fields
    let name = plugin_parameters
        .get("name")
        .and_then(StringOrList::as_string) // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
        .expect("required plugin.yml attribute not given: name (see package.metadata.waterjet.name or set CARGO_PKG_NAME)");
    plugin_yml.push_str(format!("name: {}\n", name).as_str());

    let main = format!("waterjet.com.github.{}\n", name);
    plugin_yml.push_str(format!("main: {}", main).as_str());

    let version = plugin_parameters
        .get("version")
        .and_then(StringOrList::as_string)
        .map(|s| s.lines().join_with(" ").to_string())
        .expect("required plugin.yml attribute not given: version (see package.metadata.waterjet.version or set CARGO_PKG_VERSION)");
    plugin_yml.push_str(format!("version: {}\n", version).as_str());

    // optional automatic fields
    if let Some(authors) = plugin_parameters
        .get("authors")
        .cloned()
        .map(StringOrList::into_list)
        .as_ref()
        .map(StringOrList::to_yaml_string)
    {
        plugin_yml.push_str(format!("authors: {}\n", authors).as_str());
    }

    if let Some(description) = plugin_parameters
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

    if let Some(website) = plugin_parameters
        .get("website")
        .and_then(StringOrList::as_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("website: {}\n", website).as_str());
    }

    if let Some(api_version) = plugin_parameters
        .get("api-version")
        .and_then(StringOrList::as_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("api-version: {}\n", api_version).as_str());
    }

    if let Some(prefix) = plugin_parameters
        .get("prefix")
        .and_then(StringOrList::as_string)
        // remove newlines
        .map(|s| s.lines().join_with(" ").to_string())
    {
        plugin_yml.push_str(format!("prefix: {}\n", prefix).as_str());
    }

    if let Some(loadbefore) = plugin_parameters
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

    if let Some(depend) = plugin_parameters
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

    if let Some(softdepend) = plugin_parameters
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

    if let Some(extra) = plugin_parameters
        .get("extra")
        .and_then(StringOrList::as_string)
    {
        plugin_yml.push_str(&extra);
    }

    Ok((name, plugin_yml))
}
