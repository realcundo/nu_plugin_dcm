#[macro_use]
extern crate assert_matches;

use std::path::PathBuf;

use nu_plugin::Plugin;
use nu_plugin_dcm::plugin::DcmPlugin;

use nu_protocol::{ReturnSuccess, UntaggedValue, Value};
use nu_test_support::value::{nothing, row, string};

use indexmap::indexmap;

fn get_asset_filename(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(filename)
}

pub fn filepath(input: impl Into<PathBuf>) -> Value {
    UntaggedValue::filepath(input.into()).into_untagged_value()
}

#[test]
fn no_input_without_errors() {
    let mut p = DcmPlugin::default();
    let actual = p.filter(nothing());

    // TODO compare with expected
    assert!(actual.is_err());
}

#[test]
fn no_input_with_silent_errors() {
    let mut p = DcmPlugin {
        silent_errors: true,
        ..Default::default()
    };

    let actual = p.filter(nothing());

    // TODO can't compare directly because ReturnSuccess doesn't impl == yet.
    assert_matches!(&actual.unwrap()[..], [Ok(ReturnSuccess::Value(actual_value))] if actual_value == &nothing());
}

#[test]
fn read_explicit_vr_big_endian_preamble() {
    let filename = get_asset_filename("ExplicitVRBigEndian-Preamble.dcm");

    let mut p = DcmPlugin::default();
    let actual = p.filter(filepath(filename));

    let expected = row(indexmap! {
        "TransferSyntax".to_string() => string("1.2.840.10008.1.2.2"),
        "MediaStorageSOPClassUID".to_string() => string("1.2.840.10008.5.1.4.1.1.2"),
        "MediaStorageSOPInstanceUID".to_string() => string("1.2.3"),
        "PatientName".to_string() => string("ExplicitVRBigEndian-Preamble"),

    });

    // TODO can't compare directly because ReturnSuccess doesn't impl == yet.
    let actual = actual.unwrap()[0].as_ref().unwrap().raw_value().unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn read_explicit_vr_little_endian_preamble() {
    let filename = get_asset_filename("ExplicitVRLittleEndian-Preamble.dcm");

    let mut p = DcmPlugin::default();
    let actual = p.filter(filepath(filename));

    let expected = row(indexmap! {
        "TransferSyntax".to_string() => string("1.2.840.10008.1.2.1"),
        "MediaStorageSOPClassUID".to_string() => string("1.2.840.10008.5.1.4.1.1.2"),
        "MediaStorageSOPInstanceUID".to_string() => string("1.2.3"),
        "PatientName".to_string() => string("ExplicitVRLittleEndian-Preamble"),

    });

    // TODO can't compare directly because ReturnSuccess doesn't impl == yet.
    let actual = actual.unwrap()[0].as_ref().unwrap().raw_value().unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn read_implicit_vr_little_endian_preamble() {
    let filename = get_asset_filename("ImplicitVRLittleEndian-Preamble.dcm");

    let mut p = DcmPlugin::default();
    let actual = p.filter(filepath(filename));

    let expected = row(indexmap! {
        "TransferSyntax".to_string() => string("1.2.840.10008.1.2"),
        "MediaStorageSOPClassUID".to_string() => string("1.2.840.10008.5.1.4.1.1.2"),
        "MediaStorageSOPInstanceUID".to_string() => string("1.2.3"),
        "PatientName".to_string() => string("ImplicitVRLittleEndian-Preamble"),

    });

    // TODO can't compare directly because ReturnSuccess doesn't impl == yet.
    let actual = actual.unwrap()[0].as_ref().unwrap().raw_value().unwrap();
    assert_eq!(actual, expected);
}
