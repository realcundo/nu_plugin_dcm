use std::env;

use nu_plugin_dcm::plugin::{DcmPlugin, DcmPluginCommand};
use nu_protocol::LabeledError;

use nu_protocol::{Span, Value};
use test_utils::{filepath, get_asset_path, get_string_by_cell_path};

mod test_utils;

#[test]
fn no_input_without_errors() {
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let actual = cmd.run_filter(&p, current_dir, &Value::test_nothing(), None, None);

    let expected =
        Err(LabeledError::new("Unrecognized type in stream")
            .with_label("'dcm' expects a string (filepath), binary, or column path", Span::test_data()));

    assert_eq!(actual, expected);
}

#[test]
fn read_explicit_vr_big_endian_preamble() {
    let filename = get_asset_path("ExplicitVRBigEndian-Preamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);
    let actual_value = actual.unwrap();

    assert_eq!(get_string_by_cell_path(&actual_value, "TransferSyntax"), "1.2.840.10008.1.2.2");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&actual_value, "PatientName"), "ExplicitVRBigEndian-Preamble");
}

#[test]
fn read_explicit_vr_little_endian_preamble() {
    let filename = get_asset_path("ExplicitVRLittleEndian-Preamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);
    let actual_value = actual.unwrap();

    assert_eq!(get_string_by_cell_path(&actual_value, "TransferSyntax"), "1.2.840.10008.1.2.1");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&actual_value, "PatientName"), "ExplicitVRLittleEndian-Preamble");
}

#[test]
fn read_implicit_vr_little_endian_preamble() {
    let filename = get_asset_path("ImplicitVRLittleEndian-Preamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;
    let actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);
    let actual_value = actual.unwrap();

    assert_eq!(get_string_by_cell_path(&actual_value, "TransferSyntax"), "1.2.840.10008.1.2");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&actual_value, "PatientName"), "ImplicitVRLittleEndian-Preamble");
}

#[test]
#[ignore]
fn read_explicit_vr_big_endian_no_preamble() {
    let filename = get_asset_path("ExplicitVRBigEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;
    let _actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    todo!()
}

#[test]
#[ignore]
fn read_explicit_vr_little_endian_no_preamble() {
    let filename = get_asset_path("ExplicitVRLittleEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;
    let _actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    todo!()
}

#[test]
#[ignore]
fn read_implicit_vr_little_endian_no_preamble() {
    let filename = get_asset_path("ImplicitVRLittleEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;
    let _actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    todo!()
}
