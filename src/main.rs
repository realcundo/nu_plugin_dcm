use dicom::object as dicom_object;
use dicom::object::StandardDataDictionary;
use nu_errors::ShellError;
use nu_plugin::{serve_plugin, Plugin};
use nu_protocol::{Primitive, ReturnSuccess, Signature, TaggedDictBuilder, Value};

use crate::meta::make_row_from_dicom_metadata;

mod convert;
mod dcm;
mod meta;

impl Plugin for dcm::DicomDump<'_, '_> {
    fn config(&mut self) -> Result<nu_protocol::Signature, nu_errors::ShellError> {
        // this describes the plugin command line options
        Ok(Signature::build("dcm").desc("Parse Dicom object").filter())
    }

    fn filter(
        &mut self,
        input: nu_protocol::Value,
    ) -> Result<Vec<nu_protocol::ReturnValue>, nu_errors::ShellError> {
        match &input.value {
            // TODO not only process filenames but also binary data
            nu_protocol::UntaggedValue::Primitive(Primitive::FilePath(path)) => {
                let obj = dicom_object::open_file(path).unwrap();

                let data_row = Value::new(self.make_row_from_dicom_object(&obj), input.tag());
                let meta_row = Value::new(make_row_from_dicom_metadata(obj.meta()), input.tag());

                // TODO flatten?
                let mut d = TaggedDictBuilder::with_capacity(input.tag(), 2);
                d.insert_value("metadata".to_string(), meta_row);
                d.insert_value("data".to_string(), data_row);

                Ok(vec![ReturnSuccess::value(d.into_value())])
            }
            _ => Err(ShellError::labeled_error(
                "Unrecognized type in stream",
                "'dcm' expects a filepath column",
                input.tag.span,
            )),
        }
    }
}

fn main() {
    let dict = StandardDataDictionary::default();

    let mut dumper = dcm::DicomDump { dictionary: &dict };

    // cargo install --path .
    // echo $files | merge { echo $files.name | dcm | get data | select Modality PixelSpacing.0 PixelSpacing.1 } | sort-by Modality name
    serve_plugin(&mut dumper);
}
