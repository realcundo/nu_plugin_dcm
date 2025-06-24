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

pub fn get_asset_base_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("assets")
}

pub fn get_asset_path(filename: &str) -> PathBuf {
    get_asset_base_path().join(filename)
}
