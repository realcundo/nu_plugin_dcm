use crate::meta::make_row_from_dicom_metadata;

use dicom::object::StandardDataDictionary;
use dicom::object::{self as dicom_object, DefaultDicomObject};
use nu_errors::ShellError;
use nu_plugin::{serve_plugin, Plugin};
use nu_protocol::{
    CallInfo, ColumnPath, Primitive, ReturnSuccess, ReturnValue, Signature, SyntaxShape,
    TaggedDictBuilder, UntaggedValue, Value,
};
use nu_value_ext::get_data_by_column_path;

mod convert;
mod dcm;
mod meta;

#[derive(Default)]
struct DcmPlugin {
    dcm_dictionary: StandardDataDictionary,
    source_column: Option<ColumnPath>,
}

impl Plugin for DcmPlugin {
    fn config(&mut self) -> Result<Signature, ShellError> {
        Ok(Signature::build("dcm")
            .desc("Parse Dicom object")
            .optional(
                "column",
                SyntaxShape::ColumnPath,
                "Column name to use as Dicom source",
            )
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        // nu make sure the num of args and type is correct
        if let Some(column) = call_info.args.nth(0) {
            if let UntaggedValue::Primitive(Primitive::ColumnPath(column)) = &column.value {
                self.source_column = Some(column.clone());
            }
        }

        Ok(vec![])
    }

    fn filter(&mut self, input: Value) -> Result<Vec<ReturnValue>, ShellError> {
        // use source column if known
        if let Some(source_column) = &self.source_column {
            // FIXME error handling
            let value = get_data_by_column_path(&input, source_column, |_v, _p, e| e)?;

            match &value.value {
                UntaggedValue::Primitive(_) => self.process_value(&input, &value),
                UntaggedValue::Table(ref t) => {
                    let mut result = Vec::with_capacity(t.len());

                    for e in t {
                        result.extend(self.process_value(&input, e)?);
                    }

                    Ok(result)
                }
                UntaggedValue::Row(_) => todo!("get_data_by_column_path row branch"),
                UntaggedValue::Error(_) => todo!("get_data_by_column_path error branch"),
                UntaggedValue::Block(_) => todo!("get_data_by_column_path block branch"),
            }
        } else {
            // expect a primitive value if column is not known
            self.process_value(&input, &input)
        }
    }
}

impl DcmPlugin {
    fn process_value(&self, input: &Value, value: &Value) -> Result<Vec<ReturnValue>, ShellError> {
        // TODO not only process filenames but also binary data
        match &value.value {
            UntaggedValue::Primitive(Primitive::FilePath(path)) => {
                let obj = dicom_object::open_file(path).unwrap();
                self.process_dicom_object(input, obj)
            }
            UntaggedValue::Primitive(Primitive::String(path_as_string)) => {
                let obj = dicom_object::open_file(path_as_string).unwrap();
                self.process_dicom_object(input, obj)
            }
            _ => Err(ShellError::labeled_error(
                "Unrecognized type in stream",
                "'dcm' expects a filepath or a string",
                input.tag.span,
            )),
        }
    }

    fn process_dicom_object(
        &self,
        input: &Value,
        obj: DefaultDicomObject,
    ) -> Result<Vec<ReturnValue>, ShellError> {
        let dcm_dumper = dcm::DicomDump {
            dcm_dictionary: &self.dcm_dictionary,
        };

        let data_row = Value::new(dcm_dumper.make_row_from_dicom_object(&obj), input.tag());
        let meta_row = Value::new(make_row_from_dicom_metadata(obj.meta()), input.tag());

        // TODO flatten?
        let mut d = TaggedDictBuilder::with_capacity(input.tag(), 2);
        d.insert_value("metadata".to_string(), meta_row);
        d.insert_value("data".to_string(), data_row);

        Ok(vec![ReturnSuccess::value(d.into_value())])
    }
}

fn main() {
    let mut plugin = DcmPlugin::default();

    // cargo install --path .
    // echo $files | merge { echo $files.name | dcm | get data | select Modality PixelSpacing.0 PixelSpacing.1 } | sort-by Modality name
    serve_plugin(&mut plugin);
}
