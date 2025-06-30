use std::path::PathBuf;

use nu_plugin_dcm::plugin::DcmPlugin;
use nu_plugin_test_support::PluginTest;
use nu_protocol::{Span, Value};

#[allow(dead_code)]
pub fn get_asset_base_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
}

#[allow(dead_code)]
pub fn get_asset_path(filename: &str) -> PathBuf {
    get_asset_base_path().join(filename)
}

#[allow(dead_code)]
pub fn filepath(input: impl Into<PathBuf>) -> Value {
    let input: PathBuf = input.into();

    Value::test_string(
        input
            .as_os_str()
            .to_str()
            .unwrap(),
    )
}

#[allow(dead_code)]
pub fn setup_plugin_for_test(nu_commands: Vec<Box<dyn nu_protocol::engine::Command>>) -> Result<PluginTest, nu_protocol::ShellError> {
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    for nc in nu_commands {
        plugin_test.add_decl(nc)?;
    }

    plugin_test.add_decl(Box::new(nu_command::Open))?;
    plugin_test.add_decl(Box::new(nu_command::FromJson))?;

    plugin_test
        .engine_state_mut()
        .add_env_var("PWD".to_string(), Value::string(get_asset_base_path().to_string_lossy(), Span::test_data()));

    Ok(plugin_test)
}

/// Accesses a nested `Value` using a cell path string and panics on failure.
///
/// This is a private helper function for tests to reduce boilerplate.
fn get_value_by_cell_path(
    value: &Value,
    path: &str,
) -> Value {
    use nu_protocol::ast::PathMember;

    // split by `.`
    let cell_path: Vec<PathMember> = path
        .split('.')
        .map(|part| {
            if let Ok(idx) = part.parse::<usize>() {
                PathMember::int(idx, false, Span::test_data())
            } else {
                PathMember::string(part.to_string(), false, nu_protocol::casing::Casing::Sensitive, Span::test_data())
            }
        })
        .collect();

    // `follow_cell_path` requires an owned value, so we clone it.
    value
        .clone()
        .follow_cell_path(&cell_path)
        .unwrap_or_else(|e| panic!("Failed to follow cell path '{}': {}", path, e))
        .into_owned()
}

/// Asserts that the value at `path` is a string and returns it. Panics on failure.
#[allow(dead_code)]
pub fn get_string_by_cell_path(
    value: &Value,
    path: &str,
) -> String {
    let result_value = get_value_by_cell_path(value, path);
    result_value
        .as_str()
        .unwrap_or_else(|e| panic!("Expected string at path '{}', but found '{}'. Error: {}", path, result_value.get_type(), e))
        .to_owned()
}

/// Asserts that the value at `path` is a list of strings and returns it. Panics on failure.
#[allow(dead_code)]
pub fn get_string_list_by_cell_path(
    value: &Value,
    path: &str,
) -> Vec<String> {
    let result_value = get_value_by_cell_path(value, path);
    result_value
        .as_list()
        .and_then(|list| {
            list.iter()
                .map(|v| {
                    v.as_str()
                        .map(|s| s.to_string())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_else(|e| panic!("Expected list<string> at path '{}', but found '{}'. Error: {}", path, result_value.get_type(), e))
}

/// Asserts that the value at `path` is an int and returns it. Panics on failure.
#[allow(dead_code)]
pub fn get_int_by_cell_path(
    value: &Value,
    path: &str,
) -> i64 {
    let result_value = get_value_by_cell_path(value, path);
    result_value
        .as_int()
        .unwrap_or_else(|e| panic!("Expected int at path '{}', but found '{}'. Error: {}", path, result_value.get_type(), e))
}

/// Asserts that the value at `path` is a bool and returns it. Panics on failure.
#[allow(dead_code)]
pub fn get_bool_by_cell_path(
    value: &Value,
    path: &str,
) -> bool {
    let result_value = get_value_by_cell_path(value, path);
    result_value
        .as_bool()
        .unwrap_or_else(|e| panic!("Expected bool at path '{}', but found '{}'. Error: {}", path, result_value.get_type(), e))
}

/// Asserts that the value at `path` is a float and returns it. Panics on failure.
#[allow(dead_code)]
pub fn get_float_by_cell_path(
    value: &Value,
    path: &str,
) -> f64 {
    let result_value = get_value_by_cell_path(value, path);
    result_value
        .as_float()
        .unwrap_or_else(|e| panic!("Expected float at path '{}', but found '{}'. Error: {}", path, result_value.get_type(), e))
}

/// Asserts that the value at `path` is nothing. Panics on failure.
#[allow(dead_code)]
pub fn assert_nothing_by_cell_path(
    value: &Value,
    path: &str,
) {
    let result_value = get_value_by_cell_path(value, path);
    if !matches!(result_value, Value::Nothing { .. }) {
        panic!("Expected nothing at path '{}', but found '{}' with value: {:?}", path, result_value.get_type(), result_value);
    }
}
