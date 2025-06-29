use std::env;

use nu_plugin_dcm::plugin::{DcmPlugin, DcmPluginCommand};
use nu_protocol::{IntoPipelineData, LabeledError};

use nu_protocol::{Span, Value};
use test_case::test_case;
use test_utils::{filepath, get_asset_path, get_string_by_cell_path};

mod test_utils;

#[test]
fn no_input_without_errors() {
    let current_dir = Ok(env::current_dir().unwrap());

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let value = Value::nothing(Span::test_data());
    let actual = cmd.process_pipeline_data(p, current_dir, None, &value.span(), value.into_pipeline_data());

    let expected_err =
        LabeledError::new("Unrecognized type in stream").with_label("'dcm' expects a string (filepath), binary, or column path", Span::test_data());

    assert_eq!(actual.unwrap_err(), expected_err);
}

#[test_case(
    "ExplicitVRBigEndian-Preamble.dcm",
    "1.2.840.10008.1.2.2",
    "ExplicitVRBigEndian-Preamble";
    "read_explicit_vr_big_endian_preamble")]
#[test_case(
    "ExplicitVRLittleEndian-Preamble.dcm",
    "1.2.840.10008.1.2.1",
    "ExplicitVRLittleEndian-Preamble";
    "read_explicit_vr_little_endian_preamble")]
#[test_case(
    "ImplicitVRLittleEndian-Preamble.dcm",
    "1.2.840.10008.1.2",
    "ImplicitVRLittleEndian-Preamble";
    "read_implicit_vr_little_endian_preamble")]
fn read_dcm_file(
    filename: &str,
    transfer_syntax: &str,
    patient_name: &str,
) {
    let filename = get_asset_path(filename);
    let current_dir = Ok(env::current_dir().unwrap());

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let value = filepath(filename);
    let actual = cmd.process_pipeline_data(p, current_dir, None, &value.span(), value.into_pipeline_data());
    let actual_value = actual
        .unwrap()
        .into_value(Span::test_data())
        .unwrap();

    assert_eq!(get_string_by_cell_path(&actual_value, "TransferSyntax"), transfer_syntax);
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");
    assert_eq!(get_string_by_cell_path(&actual_value, "MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&actual_value, "PatientName"), patient_name);
}

#[test]
#[ignore]
fn read_explicit_vr_big_endian_no_preamble() {
    let filename = get_asset_path("ExplicitVRBigEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let value = filepath(filename);
    let actual = cmd.process_pipeline_data(p, current_dir, None, &value.span(), value.into_pipeline_data());
    let _actual_value = actual
        .unwrap()
        .into_value(Span::test_data())
        .unwrap();

    todo!()
}

#[test]
#[ignore]
fn read_explicit_vr_little_endian_no_preamble() {
    let filename = get_asset_path("ExplicitVRLittleEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let value = filepath(filename);
    let actual = cmd.process_pipeline_data(p, current_dir, None, &value.span(), value.into_pipeline_data());
    let _actual_value = actual
        .unwrap()
        .into_value(Span::test_data())
        .unwrap();

    todo!()
}

#[test]
#[ignore]
fn read_implicit_vr_little_endian_no_preamble() {
    let filename = get_asset_path("ImplicitVRLittleEndian-NoPreamble.dcm");
    let current_dir = Ok(env::current_dir().unwrap());

    let p = DcmPlugin::default();
    let cmd = DcmPluginCommand;

    let value = filepath(filename);
    let actual = cmd.process_pipeline_data(p, current_dir, None, &value.span(), value.into_pipeline_data());
    let _actual_value = actual
        .unwrap()
        .into_value(Span::test_data())
        .unwrap();

    todo!()
}
