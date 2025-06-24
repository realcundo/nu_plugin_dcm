use nu_protocol::Span;
use test_utils::{get_string_by_cell_path, setup_plugin_for_test};

mod test_utils;

const TEST_SPAN: Span = Span::test_data();

#[test]
fn read_dicomweb_list() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![Box::new(nu_command::Open), Box::new(nu_command::FromJson)])?;

    let result = plugin_test.eval("open dicomweb-example.json | dcm")?;
    let result = result.into_value(TEST_SPAN)?;

    {
        assert_eq!(get_string_by_cell_path(&result, "0.StudyInstanceUID"), "1.2.392.200036.9116.2.2.2.1762893313.1029997326.945873");
        assert_eq!(get_string_by_cell_path(&result, "0.PatientName"), "Wang^XiaoDong");
        assert_eq!(get_string_by_cell_path(&result, "0.OtherPatientIDsSequence.0.PatientID"), "54321");
        assert_eq!(get_string_by_cell_path(&result, "0.OtherPatientIDsSequence.0.IssuerOfPatientID"), "Hospital B");
        assert_eq!(get_string_by_cell_path(&result, "0.OtherPatientIDsSequence.1.PatientID"), "24680");
        assert_eq!(get_string_by_cell_path(&result, "0.OtherPatientIDsSequence.1.IssuerOfPatientID"), "Hospital C");
    }

    {
        assert_eq!(get_string_by_cell_path(&result, "1.StudyInstanceUID"), "1.2.392.200036.9116.2.2.2.2162893313.1029997326.945876");
        assert_eq!(get_string_by_cell_path(&result, "1.PatientName"), "Wang^XiaoDong");
        assert_eq!(get_string_by_cell_path(&result, "1.OtherPatientIDsSequence.0.PatientID"), "54321");
        assert_eq!(get_string_by_cell_path(&result, "1.OtherPatientIDsSequence.0.IssuerOfPatientID"), "Hospital B2");
        assert_eq!(get_string_by_cell_path(&result, "1.OtherPatientIDsSequence.1.PatientID"), "24680");
        assert_eq!(get_string_by_cell_path(&result, "1.OtherPatientIDsSequence.1.IssuerOfPatientID"), "Hospital C2");
    }

    Ok(())
}
