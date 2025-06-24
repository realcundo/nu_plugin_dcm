use nu_protocol::Span;
use test_utils::setup_plugin_for_test;

mod test_utils;

#[test]
fn read_dicomweb_list() -> Result<(), nu_protocol::ShellError> {
    let mut plugin_test = setup_plugin_for_test(vec![
        Box::new(nu_command::Open),
        Box::new(nu_command::FromJson),
    ])?;

    let result = plugin_test.eval("open dicomweb-example.json")?;

    let result = result.into_value(Span::test_data())?;

    result.as_list()?;
    Ok(())
}
