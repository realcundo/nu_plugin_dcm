#[macro_use]
extern crate assert_matches;

use nu_plugin::Plugin;
use nu_plugin_dcm::plugin::DcmPlugin;

use nu_protocol::ReturnSuccess;
use nu_test_support::value::nothing;

#[test]
fn test_no_input_without_errors() {
    let mut p = DcmPlugin::default();
    let actual = p.filter(nothing());

    // TODO compare with expected
    assert!(actual.is_err());
}

#[test]
fn test_no_input_with_silent_errors() {
    let mut p = DcmPlugin {
        silent_errors: true,
        ..Default::default()
    };

    let actual = p.filter(nothing());

    // TODO can't compare directly because ReturnSuccess doesn't impl == yet.
    assert_matches!(&actual.unwrap()[..], [Ok(ReturnSuccess::Value(expected))] if expected == &nothing());
}
