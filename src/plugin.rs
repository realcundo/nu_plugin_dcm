use std::io::Cursor;

use crate::meta::make_row_from_dicom_metadata;
use crate::reader::{read_dcm_file, read_dcm_stream};

use crate::dcm;
use dicom::object::DefaultDicomObject;
use dicom::object::StandardDataDictionary;
use indexmap::IndexMap;
use nu_plugin::{LabeledError, Plugin};
use nu_protocol::ast::CellPath;
use nu_protocol::{Category, Signature, Span, Spanned, SyntaxShape, Value};

#[derive(Default)]
pub struct DcmPlugin {
    pub dcm_dictionary: StandardDataDictionary,
}

impl Plugin for DcmPlugin {
    fn signature(&self) -> Vec<Signature> {
        vec![Signature::build("dcm")
            .desc("Parse Dicom object from file or binary data. Invalid Dicom objects are reported as errors and excluded from the output.")
            .switch(
                "silent-errors",
                "For each input row generate an output row. If Dicom cannot be read, empty row is output. This makes sure that the number of input and output rows is identical. Useful for merging tables.",
                Some('s')
            )
            .optional(
                "column",
                SyntaxShape::CellPath,
                "Optional column name to use as Dicom source",
            )
            .category(Category::Filters)
        ]
    }

    fn run(
        &mut self,
        _name: &str,
        call: &nu_plugin::EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        // parse args, nu makes sure the num of args and type is correct
        let column = call.nth(0);

        let source_column = if let Some(Value::CellPath { val, span: _ }) = &column {
            Some(val.clone())
        } else {
            None
        };

        // run
        let result = self.run_filter(input, source_column);

        // TODO are silent errors still useful? Figure out how to deal with errors here vs errors in the CellPath list
        let silent_errors = call.has_flag("silent-errors");
        match (silent_errors, &result) {
            // ok or errors should not be silenced, return as is
            (_, Ok(_)) | (false, Err(_)) => result,

            // found an error and we should silence it
            (true, Err(_error)) => Ok(Value::Nothing { span: call.head }),
        }
    }
}

impl DcmPlugin {
    pub fn run_filter(
        &mut self,
        value: &Value,
        source_column: Option<CellPath>,
    ) -> Result<Value, LabeledError> {
        // use source column if known
        if let Some(source_column) = source_column {
            // TODO is it possible without cloning?
            let value = value.clone().follow_cell_path(&source_column.members)?;

            // AFAIK a list, process_value will handle it
            self.process_value(&value)
        } else {
            // expect a primitive value if column is not known
            self.process_value(value)
        }
    }

    fn process_value(&self, value: &Value) -> Result<Value, LabeledError> {
        match &value {
            Value::String { val, span } => {
                let obj = read_dcm_file(val).map_err(|e| LabeledError {
                    label: "'dcm' expects valid Dicom binary data".to_owned(),
                    msg: format!("{} [file {}]", e, val),
                    span: Some(*span),
                })?;

                self.process_dicom_object(span, obj)
            }
            Value::Binary { val, span } => {
                let cursor = Cursor::new(val);
                let obj = read_dcm_stream(cursor).map_err(|e| LabeledError {
                    label: "'dcm' expects valid Dicom binary data".to_owned(),
                    msg: e.to_string(),
                    span: Some(*span),
                })?;

                self.process_dicom_object(span, obj)
            }
            Value::List { vals, span } => {
                // use either a dicom result or an error for each input element
                // TODO respect silent-errors flag?
                let result: Vec<Value> = vals
                    .iter()
                    .map(|v| {
                        self.process_value(v)
                            .unwrap_or_else(|e| Value::Error { error: e.into() })
                    })
                    .collect();

                Ok(Value::List {
                    vals: result,
                    span: *span,
                })
            }
            _ => Err(LabeledError {
                label: "Unrecognized type in stream".to_owned(),
                msg: "'dcm' expects a string (filepath), binary, or column path".to_owned(),
                span: value.span().ok(),
            }),
        }
    }

    fn process_dicom_object(
        &self,
        span: &Span,
        obj: DefaultDicomObject,
    ) -> Result<Value, LabeledError> {
        let dcm_dumper = dcm::DicomDump {
            dcm_dictionary: &self.dcm_dictionary,
        };

        // dump both metadata and data into a single table
        let mut index_map = IndexMap::with_capacity(1000);
        make_row_from_dicom_metadata(span, &mut index_map, obj.meta());
        dcm_dumper.make_row_from_dicom_object(span, &mut index_map, &obj);

        let index_map = Spanned {
            item: index_map,
            span: *span,
        };

        Ok(Value::from(index_map))
    }
}
