use std::path::PathBuf;

use nu_plugin_dcm::plugin::DcmPlugin;
use nu_plugin_test_support::PluginTest;
use nu_protocol::{Span, Value};

#[macro_export]
macro_rules! assert_dicom_field {
    ($record:expr, $field:expr, $expected:expr) => {
        assert_eq!(
            $record.get($field),
            Some(&Value::string($expected, Span::test_data()))
        );
    };
}

pub fn get_asset_base_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
}

pub fn get_asset_path(filename: &str) -> PathBuf {
    get_asset_base_path().join(filename)
}

pub fn filepath(input: impl Into<PathBuf>) -> Value {
    let input: PathBuf = input.into();

    Value::test_string(input.as_os_str().to_str().unwrap())
}

pub fn setup_plugin_for_test(
    nu_commands: Vec<Box<dyn nu_protocol::engine::Command>>,
) -> Result<PluginTest, nu_protocol::ShellError> {
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    for nc in nu_commands {
        plugin_test.add_decl(nc)?;
    }

    plugin_test.add_decl(Box::new(nu_command::Open))?;
    plugin_test.add_decl(Box::new(nu_command::FromJson))?;

    plugin_test.engine_state_mut().add_env_var(
        "PWD".to_string(),
        Value::string(get_asset_base_path().to_string_lossy(), Span::test_data()),
    );

    Ok(plugin_test)
}
