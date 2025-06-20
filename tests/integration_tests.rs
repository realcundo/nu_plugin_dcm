use nu_plugin_dcm::plugin::DcmPlugin;
use nu_plugin_test_support::PluginTest;
use nu_protocol::{record, IntoPipelineData, Span, Value};
use std::path::PathBuf;

#[macro_export]
macro_rules! assert_dicom_field {
    ($record:expr, $field:expr, $expected:expr) => {
        assert_eq!(
            $record.get($field),
            Some(&Value::string($expected, Span::test_data()))
        );
    };
}

fn get_asset_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
        .join(filename)
}

fn create_binary_value(filename: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let test_file = get_asset_path(filename);
    let test_data = std::fs::read(&test_file)?;
    Ok(Value::binary(test_data, Span::test_data()))
}

fn create_file_record_value<S: AsRef<str>>(filename: S) -> Value {
    let test_file = get_asset_path(filename.as_ref());

    Value::record(
        record! {
            "name" => Value::string(test_file.to_string_lossy(), Span::test_data()),
            "type" => Value::string("file", Span::test_data()),
        },
        Span::test_data(),
    )
}

/// Simulate `open file | dcm`
#[test]
fn test_scalar_binary_input() -> Result<(), Box<dyn std::error::Error>> {
    // Test with direct binary input using eval_with method
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    let binary_value = create_binary_value("ExplicitVRLittleEndian-Preamble.dcm")?;

    // Use eval_with to pass the binary data directly to dcm
    let pipeline_data = plugin_test.eval_with("dcm", binary_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    let record = result.as_record()?;

    assert_dicom_field!(record, "TransferSyntax", "1.2.840.10008.1.2.1");
    assert_dicom_field!(record, "PatientName", "ExplicitVRLittleEndian-Preamble");

    Ok(())
}

/// Simulate `[file1, file2] | each { |f| open $f } | dcm`
#[test]
fn test_vector_binary_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    let test_files = vec![
        "ExplicitVRLittleEndian-Preamble.dcm",
        "ExplicitVRBigEndian-Preamble.dcm",
        "ImplicitVRLittleEndian-Preamble.dcm",
    ];

    // Create a list of binary values (simulating what multiple 'open' commands would produce)
    let binary_list = test_files
        .into_iter()
        .map(create_binary_value)
        .collect::<Result<Vec<_>, _>>()?;

    let list_value = Value::list(binary_list, Span::test_data());

    let pipeline_data = plugin_test.eval_with("dcm", list_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    let records = result.as_list()?;

    assert_eq!(records.len(), 3, "Should process all 3 files");

    for val in records {
        let record = val.as_record()?;

        assert!(record.get("TransferSyntax").is_some());
        assert!(record.get("PatientName").is_some());
    }

    Ok(())
}

/// Simulate `"file.dcm" | dcm` (string file path input)
#[test]
fn test_string_file_path_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    let test_file = get_asset_path("ExplicitVRBigEndian-Preamble.dcm");
    let file_path_value = Value::string(test_file.to_string_lossy(), Span::test_data());

    let pipeline_data = plugin_test.eval_with("dcm", file_path_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    let record = result.as_record()?;

    assert_dicom_field!(record, "TransferSyntax", "1.2.840.10008.1.2.2");
    assert_dicom_field!(record, "PatientName", "ExplicitVRBigEndian-Preamble");

    Ok(())
}
/// Simulate `ls *.dcm | dcm name` matching a single file
#[test]
fn test_scalar_record_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    // Create a record that simulates what 'ls' would produce
    let file_record = create_file_record_value("ImplicitVRLittleEndian-Preamble.dcm");

    // Test with column path 'name'
    let pipeline_data = plugin_test.eval_with("dcm name", file_record.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    let record = result.as_record()?;

    assert_dicom_field!(record, "TransferSyntax", "1.2.840.10008.1.2");
    assert_dicom_field!(record, "PatientName", "ImplicitVRLittleEndian-Preamble");

    Ok(())
}

/// Simulate `ls *.dcm | dcm name` with multiple matching files
#[test]
fn test_vector_record_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = PluginTest::new("dcm", DcmPlugin::default().into())?;

    // Create multiple records that simulate what 'ls *.dcm' would produce
    let test_files = vec![
        "ExplicitVRLittleEndian-Preamble.dcm",
        "ExplicitVRBigEndian-Preamble.dcm",
        "ImplicitVRLittleEndian-Preamble.dcm",
    ];

    let file_records: Vec<Value> = test_files.iter().map(create_file_record_value).collect();

    let list_value = Value::list(file_records, Span::test_data());

    // Test with column path 'name'
    let pipeline_data = plugin_test.eval_with("dcm name", list_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    let records = result.as_list()?;

    assert_eq!(records.len(), 3, "Should process all 3 files");

    // Verify each record has the expected fields
    for (i, val) in records.iter().enumerate() {
        let record = val.as_record()?;

        assert!(
            record.get("TransferSyntax").is_some(),
            "Record {} missing TransferSyntax",
            i
        );
        assert!(
            record.get("PatientName").is_some(),
            "Record {} missing PatientName",
            i
        );
    }

    // Verify specific patient names match the filenames
    let expected_names = vec![
        "ExplicitVRLittleEndian-Preamble",
        "ExplicitVRBigEndian-Preamble",
        "ImplicitVRLittleEndian-Preamble",
    ];

    for (i, expected_name) in expected_names.iter().enumerate() {
        let record = records[i].as_record()?;
        assert_dicom_field!(record, "PatientName", *expected_name);
    }

    Ok(())
}
