use std::io::{Cursor, Seek, SeekFrom};

use crate::meta::make_row_from_dicom_metadata;

use dicom::object::StandardDataDictionary;
use dicom::object::{self as dicom_object, DefaultDicomObject};
use indexmap::IndexMap;
use nu_errors::ShellError;
use nu_plugin::{serve_plugin, Plugin};
use nu_protocol::{
    CallInfo, ColumnPath, Primitive, ReturnSuccess, ReturnValue, Signature, SyntaxShape,
    UntaggedValue, Value,
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
            .desc("Parse Dicom object from file or binary data")
            .optional(
                "column",
                SyntaxShape::ColumnPath,
                "Column name to use as Dicom source",
            )
            .filter())
    }

    fn begin_filter(&mut self, call_info: CallInfo) -> Result<Vec<ReturnValue>, ShellError> {
        // nu makes sure the num of args and type is correct
        if let Some(column) = call_info.args.nth(0) {
            if let UntaggedValue::Primitive(Primitive::ColumnPath(column)) = &column.value {
                self.source_column = Some(column.clone());
            }
        }

        Ok(vec![])
    }

    fn filter(&mut self, value: Value) -> Result<Vec<ReturnValue>, ShellError> {
        let tag = value.tag();

        // use source column if known
        if let Some(source_column) = &self.source_column {
            // FIXME error handling
            let value = get_data_by_column_path(&value, source_column, |_v, _p, e| e)?;

            match &value.value {
                UntaggedValue::Primitive(_) => self.process_value(tag, &value),
                UntaggedValue::Table(ref t) => {
                    let mut result = Vec::with_capacity(t.len());

                    for e in t {
                        result.extend(self.process_value(tag.clone(), e)?);
                    }

                    Ok(result)
                }
                UntaggedValue::Row(_) => todo!("get_data_by_column_path row branch"),
                UntaggedValue::Block(_) => todo!("get_data_by_column_path block branch"),
                UntaggedValue::Error(e) => Err(e.clone()),
            }
        } else {
            // expect a primitive value if column is not known
            self.process_value(tag, &value)
        }
    }
}

impl DcmPlugin {
    fn process_value(
        &self,
        tag: nu_source::Tag,
        value: &Value,
    ) -> Result<Vec<ReturnValue>, ShellError> {
        match &value.value {
            UntaggedValue::Primitive(Primitive::FilePath(path)) => {
                let obj = dicom_object::open_file(path).map_err(|e| {
                    ShellError::labeled_error(
                        format!("{} [file {}]", e, path.to_string_lossy()),
                        "'dcm' expects a valid Dicom file",
                        tag.span,
                    )
                })?;

                self.process_dicom_object(tag, obj)
            }
            UntaggedValue::Primitive(Primitive::String(path_as_string)) => {
                let obj = dicom_object::open_file(path_as_string).map_err(|e| {
                    ShellError::labeled_error(
                        format!("{} [file {}]", e, path_as_string),
                        "'dcm' expects a valid Dicom file",
                        tag.span,
                    )
                })?;

                self.process_dicom_object(tag, obj)
            }
            UntaggedValue::Primitive(Primitive::Binary(data)) => {
                let mut cursor = Cursor::new(data);

                // FIXME don't assume the preamble
                cursor
                    .seek(SeekFrom::Start(128))
                    .ok()
                    .filter(|new_pos| *new_pos == 128) // Unexpectedly this is true even for binaries < 128 bytes long
                    .ok_or_else(|| {
                        ShellError::labeled_error(
                            "Cannot read Dicom preamble",
                            "'dcm' expects valid Dicom binary data",
                            tag.span,
                        )
                    })?;

                let obj = dicom_object::from_reader(cursor).map_err(|e| {
                    ShellError::labeled_error(
                        e.to_string(),
                        "'dcm' expects valid Dicom binary data",
                        tag.span,
                    )
                })?;

                self.process_dicom_object(tag, obj)
            }
            _ => Err(ShellError::labeled_error(
                "Unrecognized type in stream",
                "'dcm' expects a filepath, binary, string or a column path",
                tag.span,
            )),
        }
    }

    fn process_dicom_object(
        &self,
        tag: nu_source::Tag,
        obj: DefaultDicomObject,
    ) -> Result<Vec<ReturnValue>, ShellError> {
        let dcm_dumper = dcm::DicomDump {
            dcm_dictionary: &self.dcm_dictionary,
        };

        // dump both metadata and data into a single table
        let mut index_map = IndexMap::with_capacity(1000);
        make_row_from_dicom_metadata(&mut index_map, obj.meta());
        dcm_dumper.make_row_from_dicom_object(&mut index_map, &obj);

        let value = Value::new(UntaggedValue::Row(index_map.into()), tag);
        Ok(vec![Ok(ReturnSuccess::Value(value))])
    }
}

fn main() {
    let mut plugin = DcmPlugin::default();

    // cargo install --path .
    // echo $files | merge { echo $files.name | dcm | get data | select Modality PixelSpacing.0 PixelSpacing.1 } | sort-by Modality name
    serve_plugin(&mut plugin);
}
