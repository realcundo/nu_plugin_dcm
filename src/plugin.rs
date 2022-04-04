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
            .named(
                "error",
                SyntaxShape::String,
                "If an error occurs when Dicom object is parsed, the error message will be inserted in this column instead producing an error result.",
                Some('e'))
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
        let source_column = call.nth(0);
        let source_column = if let Some(Value::CellPath { val, span: _ }) = source_column {
            Some(val)
        } else {
            None
        };

        let error_column = call.get_flag_value("error");
        let error_column = if let Some(Value::String { val, span: _ }) = error_column {
            Some(val)
        } else {
            None
        };

        // run
        self.run_filter(input, source_column, error_column)
    }
}

impl DcmPlugin {
    pub fn run_filter(
        &mut self,
        value: &Value,
        source_column: Option<CellPath>,
        error_column: Option<String>,
    ) -> Result<Value, LabeledError> {
        // use source column if known
        if let Some(source_column) = source_column {
            // TODO is it possible without cloning?
            let value = value.clone().follow_cell_path(&source_column.members)?;

            // AFAIK a list, process_value will handle it
            self.process_value(&value, &error_column)
        } else {
            // expect a primitive value if column is not known
            self.process_value(value, &error_column)
        }
    }

    fn process_value(
        &self,
        value: &Value,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        let result = self.process_value_with_normal_error(value, error_column);

        // TODO better value.span().unwrap()
        match (error_column, &result) {
            (Some(error_column), Err(err)) => Ok(Value::Record {
                cols: vec![error_column.to_string()],
                vals: vec![Value::string(err.msg.to_string(), value.span().unwrap())],
                span: value.span().unwrap(),
            }),
            _ => result,
        }
    }

    fn process_value_with_normal_error(
        &self,
        value: &Value,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        match &value {
            Value::String { val, span } => {
                let obj = read_dcm_file(val).map_err(|e| LabeledError {
                    label: "'dcm' expects valid Dicom binary data".to_owned(),
                    msg: format!("{} [file {}]", e, val),
                    span: Some(*span),
                })?;

                self.process_dicom_object(span, obj, error_column)
            }
            Value::Binary { val, span } => {
                let cursor = Cursor::new(val);
                let obj = read_dcm_stream(cursor).map_err(|e| LabeledError {
                    label: "'dcm' expects valid Dicom binary data".to_owned(),
                    msg: e.to_string(),
                    span: Some(*span),
                })?;

                self.process_dicom_object(span, obj, error_column)
            }
            Value::List { vals, span } => {
                // Use either a dicom result or an error for each input element>
                let result: Vec<Value> = vals
                    .iter()
                    .map(|v| {
                        self.process_value(v, error_column)
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
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        let dcm_dumper = dcm::DicomDump {
            dcm_dictionary: &self.dcm_dictionary,
        };

        let mut index_map = IndexMap::with_capacity(1000);

        // make sure that when --error is used, the column always exists
        if let Some(error_column) = error_column {
            index_map.insert(
                error_column.to_string(),
                Value::string(String::new(), *span),
            );
        }

        // dump both metadata and data into a single table
        make_row_from_dicom_metadata(span, &mut index_map, obj.meta());
        dcm_dumper.make_row_from_dicom_object(span, &mut index_map, &obj);

        let index_map = Spanned {
            item: index_map,
            span: *span,
        };

        Ok(Value::from(index_map))
    }
}
