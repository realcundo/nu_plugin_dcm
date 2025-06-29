use nu_plugin_dcm::plugin::DcmPluginCommand;
use nu_protocol::Span;
use test_case::test_case;
use test_utils::{get_string_by_cell_path, setup_plugin_for_test};

mod test_utils;

#[test]
#[ignore]
fn test_examples() -> Result<(), nu_protocol::ShellError> {
    // TODO create an examples directory with dicom files for example testing. That said, is it worth it? Most examples don't have return values.
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::Ls)])?;

    plugin_test.test_command_examples(&DcmPluginCommand)
}

#[test_case("\"file.dcm\" | dcm"; "pass filename as string")]
#[test_case("echo \"file.dcm\" | dcm"; "echo filename as string")]
#[test_case("open --raw file.dcm | dcm"; "open stream")]
#[test_case("open --raw file.dcm | into binary | dcm"; "open binary stream")]
#[test_case("open --raw file.dcm | into binary | collect | dcm"; "open binary blob")]
fn test_command_scalar(command: &str) -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::IntoBinary)])?;

    let result = plugin_test.eval(command)?;
    let result = result.into_value(Span::test_data())?;

    assert_eq!(get_string_by_cell_path(&result, "PatientName"), "ExplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&result, "TransferSyntax"), "1.2.840.10008.1.2.1");
    assert_eq!(get_string_by_cell_path(&result, "MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");

    Ok(())
}

#[test]
fn test_command_vector_open() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::IntoBinary)])?;

    let result = plugin_test.eval("[(open --raw file.dcm | into binary), (open --raw file.dcm | into binary)] | dcm")?;

    let result = result.into_value(Span::test_data())?;

    assert_eq!(
        result
            .as_list()?
            .len(),
        2
    );

    assert_eq!(get_string_by_cell_path(&result, "0.PatientName"), "ExplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "0.MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&result, "0.TransferSyntax"), "1.2.840.10008.1.2.1");
    assert_eq!(get_string_by_cell_path(&result, "0.MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");

    assert_eq!(get_string_by_cell_path(&result, "1.PatientName"), "ExplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "1.MediaStorageSOPInstanceUID"), "1.2.3");
    assert_eq!(get_string_by_cell_path(&result, "1.TransferSyntax"), "1.2.840.10008.1.2.1");
    assert_eq!(get_string_by_cell_path(&result, "1.MediaStorageSOPClassUID"), "1.2.840.10008.5.1.4.1.1.2");

    Ok(())
}

#[test_case("ls *-Preamble.dcm | sort-by name | dcm"; "ls *.dcm")] // list of records
#[test_case("ls *-Preamble.dcm | sort-by name | select name type | dcm"; "ls *.dcm | select name")] // list of records
#[test_case("ls *-Preamble.dcm | sort-by name | get name | dcm"; "ls *.dcm | get name")] // list of names
fn test_command_ls(command: &str) -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test =
        setup_plugin_for_test(vec![Box::new(nu_command::Ls), Box::new(nu_command::SortBy), Box::new(nu_command::Get), Box::new(nu_command::Select)])?;

    let result = plugin_test.eval(command)?;
    let result = result.into_value(Span::test_data())?;

    assert_eq!(
        result
            .as_list()?
            .len(),
        3
    );

    assert_eq!(get_string_by_cell_path(&result, "0.PatientName"), "ExplicitVRBigEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "0.TransferSyntax"), "1.2.840.10008.1.2.2");

    assert_eq!(get_string_by_cell_path(&result, "1.PatientName"), "ExplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "1.TransferSyntax"), "1.2.840.10008.1.2.1");

    assert_eq!(get_string_by_cell_path(&result, "2.PatientName"), "ImplicitVRLittleEndian-Preamble");
    assert_eq!(get_string_by_cell_path(&result, "2.TransferSyntax"), "1.2.840.10008.1.2");
    Ok(())
}

#[test]
fn test_fail_on_extra_parameters() -> Result<(), Box<dyn std::error::Error>> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::Ls)])?;

    let pipeline_data = plugin_test.eval("ls *.dcm | dcm foo");

    let error = pipeline_data.unwrap_err();
    if let nu_protocol::ShellError::LabeledError(labeled_error) = error {
        assert_eq!(labeled_error.msg, "Example failed to parse"); // coming from plugin test

        let inner_error = labeled_error
            .inner
            .get(0)
            .expect("Expected an inner error");

        dbg!(&inner_error);
        assert_eq!(inner_error.msg, "Extra positional argument.");
        assert_eq!(inner_error.help, Some("Usage: dcm {flags} ".to_string()));
    } else {
        panic!("Unexpected error {:?}", error);
    }

    Ok(())
}
