use std::path::PathBuf;

use nu_plugin::LabeledError;
use nu_plugin_dcm::plugin::DcmPlugin;

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
    let mut p = DcmPlugin::default();
    let actual = p.run_filter(&Value::test_nothing(), None, None);

    let expected = Err(LabeledError {
        label: "Unrecognized type in stream".to_owned(),
        msg: "'dcm' expects a string (filepath), binary, or column path".to_owned(),
        span: Some(Span::test_data()),
    });

    assert_eq!(actual, expected);
}

#[test]
fn read_explicit_vr_big_endian_preamble() {
    let filename = get_asset_filename("ExplicitVRBigEndian-Preamble.dcm");

    let mut p = DcmPlugin::default();
    let actual = p.run_filter(&filepath(filename), None, None);

    let expected = Ok(Value::test_record(
        vec![
            "TransferSyntax",
            "MediaStorageSOPClassUID",
            "MediaStorageSOPInstanceUID",
            "PatientName",
        ],
        vec![
            Value::test_string("1.2.840.10008.1.2.2"),
            Value::test_string("1.2.840.10008.5.1.4.1.1.2"),
            Value::test_string("1.2.3"),
            Value::test_string("ExplicitVRBigEndian-Preamble"),
        ],
    ));

    assert_eq!(actual, expected);
}

#[test]
fn read_explicit_vr_little_endian_preamble() {
    let filename = get_asset_filename("ExplicitVRLittleEndian-Preamble.dcm");

    let mut p = DcmPlugin::default();
    let actual = p.run_filter(&filepath(filename), None, None);

    let expected = Ok(Value::test_record(
        vec![
            "TransferSyntax",
            "MediaStorageSOPClassUID",
            "MediaStorageSOPInstanceUID",
            "PatientName",
        ],
        vec![
            Value::test_string("1.2.840.10008.1.2.1"),
            Value::test_string("1.2.840.10008.5.1.4.1.1.2"),
            Value::test_string("1.2.3"),
            Value::test_string("ExplicitVRLittleEndian-Preamble"),
        ],
    ));

    assert_eq!(actual, expected);
}

#[test]
fn read_implicit_vr_little_endian_preamble() {
    let filename = get_asset_filename("ImplicitVRLittleEndian-Preamble.dcm");

    let mut p = DcmPlugin::default();
    let actual = p.run_filter(&filepath(filename), None, None);

    let expected = Ok(Value::test_record(
        vec![
            "TransferSyntax",
            "MediaStorageSOPClassUID",
            "MediaStorageSOPInstanceUID",
            "PatientName",
        ],
        vec![
            Value::test_string("1.2.840.10008.1.2"),
            Value::test_string("1.2.840.10008.5.1.4.1.1.2"),
            Value::test_string("1.2.3"),
            Value::test_string("ImplicitVRLittleEndian-Preamble"),
        ],
    ));

    assert_eq!(actual, expected);
}

#[test]
#[ignore]
fn read_explicit_vr_big_endian_no_preamble() {
    let filename = get_asset_filename("ExplicitVRBigEndian-NoPreamble.dcm");

    let mut p = DcmPlugin::default();
    let _actual = p.run_filter(&filepath(filename), None, None);

    todo!()
}

#[test]
#[ignore]
fn read_explicit_vr_little_endian_no_preamble() {
    let filename = get_asset_filename("ExplicitVRLittleEndian-NoPreamble.dcm");

    let mut p = DcmPlugin::default();
    let _actual = p.run_filter(&filepath(filename), None, None);

    todo!()
}

#[test]
#[ignore]
fn read_implicit_vr_little_endian_no_preamble() {
    let filename = get_asset_filename("ImplicitVRLittleEndian-NoPreamble.dcm");

    let mut p = DcmPlugin::default();
    let _actual = p.run_filter(&filepath(filename), None, None);

    todo!()
}
