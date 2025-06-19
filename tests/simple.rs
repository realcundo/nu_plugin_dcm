use std::env;
use std::path::PathBuf;

use nu_plugin_dcm::plugin::{DcmPlugin, DcmPluginCommand};
use nu_protocol::{LabeledError, Record};

use nu_protocol::{Span, Value};

fn get_asset_filename(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(filename)
}

pub fn filepath(input: impl Into<PathBuf>) -> Value {
    let input: PathBuf = input.into();

    Value::test_string(input.as_os_str().to_str().unwrap())
}

#[test]
fn no_input_without_errors() {
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let actual = cmd.run_filter(&p, current_dir, &Value::test_nothing(), None, None);

    let expected = Err(LabeledError::new("Unrecognized type in stream").with_label(
        "'dcm' expects a string (filepath), binary, or column path",
        Span::test_data(),
    ));

    assert_eq!(actual, expected);
}

#[test]
fn read_explicit_vr_big_endian_preamble() {
    let filename = get_asset_filename("ExplicitVRBigEndian-Preamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    let expected = Ok(Value::test_record(
        [
            ("TransferSyntax", "1.2.840.10008.1.2.2"),
            ("MediaStorageSOPClassUID", "1.2.840.10008.5.1.4.1.1.2"),
            ("MediaStorageSOPInstanceUID", "1.2.3"),
            ("PatientName", "ExplicitVRBigEndian-Preamble"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::string(v, Span::test_data())))
        .collect::<Record>(),
    ));

    assert_eq!(actual, expected);
}

#[test]
fn read_explicit_vr_little_endian_preamble() {
    let filename = get_asset_filename("ExplicitVRLittleEndian-Preamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    let expected = Ok(Value::test_record(
        [
            ("TransferSyntax", "1.2.840.10008.1.2.1"),
            ("MediaStorageSOPClassUID", "1.2.840.10008.5.1.4.1.1.2"),
            ("MediaStorageSOPInstanceUID", "1.2.3"),
            ("PatientName", "ExplicitVRLittleEndian-Preamble"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::string(v, Span::test_data())))
        .collect::<Record>(),
    ));

    assert_eq!(actual, expected);
}

#[test]
fn read_implicit_vr_little_endian_preamble() {
    let filename = get_asset_filename("ImplicitVRLittleEndian-Preamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;
    let actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    let expected = Ok(Value::test_record(
        [
            ("TransferSyntax", "1.2.840.10008.1.2"),
            ("MediaStorageSOPClassUID", "1.2.840.10008.5.1.4.1.1.2"),
            ("MediaStorageSOPInstanceUID", "1.2.3"),
            ("PatientName", "ImplicitVRLittleEndian-Preamble"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), Value::string(v, Span::test_data())))
        .collect::<Record>(),
    ));

    assert_eq!(actual, expected);
}

#[test]
#[ignore]
fn read_explicit_vr_big_endian_no_preamble() {
    let filename = get_asset_filename("ExplicitVRBigEndian-NoPreamble.dcm");
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
    let filename = get_asset_filename("ExplicitVRLittleEndian-NoPreamble.dcm");
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
    let filename = get_asset_filename("ImplicitVRLittleEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());
    let current_dir = current_dir.as_deref();

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;
    let _actual = cmd.run_filter(&p, current_dir, &filepath(filename), None, None);

    todo!()
}
