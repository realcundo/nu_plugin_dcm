use crate::test_utils::get_asset_base_path;
use insta::{assert_ron_snapshot, glob};
use nu_protocol::Span;
use test_utils::setup_plugin_for_test;

mod test_utils;

#[test]
fn private_assets() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open)])?;
    let asset_path = get_asset_base_path().join("private");

    glob!(asset_path, "**/*.dcm", |path| {
        let command = format!("open --raw {} | dcm", path.to_string_lossy());

        let result = plugin_test
            .eval(&command)
            .expect("failed to evaluate command")
            .into_value(Span::test_data())
            .expect("failed to convert to value");

        insta::with_settings!({sort_maps => true, snapshot_path => "snapshots/private" }, {
            assert_ron_snapshot!(result);
        });
    });

    Ok(())
}
