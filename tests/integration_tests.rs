use nu_plugin_dcm::plugin::DcmPluginCommand;
use nu_protocol::{IntoPipelineData, Span, Value, record};
use test_utils::{get_asset_path, get_string_by_cell_path, setup_plugin_for_test};

mod test_utils;

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

#[test]
#[ignore]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::Ls)])?;

    plugin_test.test_command_examples(&DcmPluginCommand)
}

#[test]
fn test_command_scalar_open() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open)])?;

    let result = plugin_test.eval("open --raw file.dcm | dcm")?;
    let result = result.into_value(Span::test_data())?;

    // TODO actually test the result
    assert!(
        result
            .as_record()
            .is_ok()
    );
    Ok(())
}

#[test]
fn test_command_vector_open() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::IntoBinary)])?;

    let result = plugin_test.eval("[(open --raw file.dcm | into binary), (open --raw file.dcm | into binary)] | dcm")?;

    let result = result.into_value(Span::test_data())?;

    // TODO actually test the result
    assert!(
        result
            .as_list()
            .is_ok()
    );
    Ok(())
}

#[test]
fn test_command_ls() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Ls)])?;

    let result = plugin_test.eval("ls *.dcm | dcm name")?;
    let result = result.into_value(Span::test_data())?;

    // TODO actually test the result
    assert!(
        result
            .as_list()
            .is_ok()
    );
    Ok(())
}

/// Simulate `open file | dcm`
#[test]
fn test_scalar_binary_input() -> Result<(), Box<dyn std::error::Error>> {
    // Test with direct binary input using eval_with method
    let mut plugin_test = setup_plugin_for_test(vec![])?;

    let binary_value = create_binary_value("ExplicitVRLittleEndian-Preamble.dcm")?;

    // Use eval_with to pass the binary data directly to dcm
    let pipeline_data = plugin_test.eval_with("dcm", binary_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    assert_eq!(get_string_by_cell_path(&result, "TransferSyntax"), "1.2.840.10008.1.2.1");
    assert_eq!(get_string_by_cell_path(&result, "PatientName"), "ExplicitVRLittleEndian-Preamble");

    Ok(())
}

/// Simulate `[file1, file2] | each { |f| open $f } | dcm`
#[test]
fn test_vector_binary_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = setup_plugin_for_test(vec![])?;

    let test_files = vec!["ExplicitVRLittleEndian-Preamble.dcm", "ExplicitVRBigEndian-Preamble.dcm", "ImplicitVRLittleEndian-Preamble.dcm"];

    // Create a list of binary values (simulating what multiple 'open' commands would produce)
    let binary_list = test_files
        .into_iter()
        .map(create_binary_value)
        .collect::<Result<Vec<_>, _>>()?;

    let list_value = Value::list(binary_list, Span::test_data());

    let pipeline_data = plugin_test.eval_with("dcm", list_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    assert_eq!(get_string_by_cell_path(&result, "0.PatientName"), "ExplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "0.TransferSyntax"), "1.2.840.10008.1.2.1");

    assert_eq!(get_string_by_cell_path(&result, "1.PatientName"), "ExplicitVRBigEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "1.TransferSyntax"), "1.2.840.10008.1.2.2");

    assert_eq!(get_string_by_cell_path(&result, "2.PatientName"), "ImplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "2.TransferSyntax"), "1.2.840.10008.1.2");

    Ok(())
}

/// Simulate `"file.dcm" | dcm` (string file path input)
#[test]
fn test_string_file_path_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = setup_plugin_for_test(vec![])?;

    let test_file = get_asset_path("ExplicitVRBigEndian-Preamble.dcm");
    let file_path_value = Value::string(test_file.to_string_lossy(), Span::test_data());

    let pipeline_data = plugin_test.eval_with("dcm", file_path_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    assert_eq!(get_string_by_cell_path(&result, "TransferSyntax"), "1.2.840.10008.1.2.2");
    assert_eq!(get_string_by_cell_path(&result, "PatientName"), "ExplicitVRBigEndian-Preamble");

    Ok(())
}
/// Simulate `ls *.dcm | dcm name` matching a single file
#[test]
fn test_scalar_record_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = setup_plugin_for_test(vec![])?;

    // Create a record that simulates what 'ls' would produce
    let file_record = create_file_record_value("ImplicitVRLittleEndian-Preamble.dcm");

    // Test with column path 'name'
    let pipeline_data = plugin_test.eval_with("dcm name", file_record.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    assert_eq!(get_string_by_cell_path(&result, "TransferSyntax"), "1.2.840.10008.1.2");
    assert_eq!(get_string_by_cell_path(&result, "PatientName"), "ImplicitVRLittleEndian-Preamble");

    Ok(())
}

/// Simulate `ls *.dcm | dcm name` with multiple matching files
#[test]
fn test_vector_record_input() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = setup_plugin_for_test(vec![])?;

    // Create multiple records that simulate what 'ls *.dcm' would produce
    let test_files = ["ExplicitVRLittleEndian-Preamble.dcm", "ExplicitVRBigEndian-Preamble.dcm", "ImplicitVRLittleEndian-Preamble.dcm"];

    let file_records: Vec<Value> = test_files
        .iter()
        .map(create_file_record_value)
        .collect();

    let list_value = Value::list(file_records, Span::test_data());

    // Test with column path 'name'
    let pipeline_data = plugin_test.eval_with("dcm name", list_value.into_pipeline_data())?;
    let result = pipeline_data.into_value(Span::test_data())?;

    assert_eq!(get_string_by_cell_path(&result, "0.PatientName"), "ExplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "0.TransferSyntax"), "1.2.840.10008.1.2.1");

    assert_eq!(get_string_by_cell_path(&result, "1.PatientName"), "ExplicitVRBigEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "1.TransferSyntax"), "1.2.840.10008.1.2.2");

    assert_eq!(get_string_by_cell_path(&result, "2.PatientName"), "ImplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "2.TransferSyntax"), "1.2.840.10008.1.2");

    Ok(())
}
