use std::io::Cursor;

use crate::meta::make_row_from_dicom_metadata;
use crate::reader::{read_dcm_file, read_dcm_stream};

use crate::dcm;
use dicom::object::DefaultDicomObject;
use dicom::object::StandardDataDictionary;
use indexmap::IndexMap;
use nu_plugin::{EngineInterface, Plugin, SimplePluginCommand};
use nu_protocol::ast::CellPath;
use nu_protocol::{Category, LabeledError, Record, Signature, Span, SyntaxShape, Value};

#[derive(Default)]
pub struct DcmPlugin {
    pub dcm_dictionary: StandardDataDictionary,
}

impl Plugin for DcmPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn commands(&self) -> Vec<Box<dyn nu_plugin::PluginCommand<Plugin = Self>>> {
        vec![Box::new(DcmPluginCommand)]
    }
}

#[derive(Default)]
pub struct DcmPluginCommand;

impl SimplePluginCommand for DcmPluginCommand {
    type Plugin = DcmPlugin;

    fn name(&self) -> &str {
        "dcm"
    }

    fn description(&self) -> &str {
        "Parse DICOM object from file or binary data. Invalid DICOM objects are reported as errors and excluded from the output."
    }

    fn signature(&self) -> Signature {
        Signature::build(nu_plugin::PluginCommand::name(self))
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
    }

    fn run(
        &self,
        plugin: &DcmPlugin,
        _engine: &EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: &Value,
    ) -> Result<Value, LabeledError> {
        // parse args, nu makes sure the num of args and type is correct
        let source_column = call.nth(0);
        let source_column = if let Some(Value::CellPath { val, .. }) = source_column {
            Some(val)
        } else {
            None
        };

        let error_column = call.get_flag_value("error");
        let error_column = if let Some(Value::String { val, .. }) = error_column {
            Some(val)
        } else {
            None
        };

        // run
        self.run_filter(plugin, input, source_column, error_column)
    }
}

impl DcmPluginCommand {
    pub fn run_filter(
        &self,
        plugin: &DcmPlugin,
        value: &Value,
        source_column: Option<CellPath>,
        error_column: Option<String>,
    ) -> Result<Value, LabeledError> {
        // use source column if known
        if let Some(source_column) = source_column {
            // TODO is it possible without cloning?
            let value = value.clone();
            let value = value.follow_cell_path(&source_column.members)?;

            // AFAIK a list, process_value will handle it
            self.process_value(plugin, &value, &error_column)
        } else {
            // expect a primitive value if column is not known
            self.process_value(plugin, value, &error_column)
        }
    }

    fn process_value(
        &self,
        plugin: &DcmPlugin,
        value: &Value,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        let result = self.process_value_with_normal_error(plugin, value, error_column);

        // TODO better value.span().unwrap()
        match (error_column, &result) {
            (Some(error_column), Err(err)) => Ok(Value::record(
                Record::from_raw_cols_vals(
                    vec![error_column.to_string()],
                    vec![Value::string(err.msg.to_string(), value.span())],
                    Span::unknown(),
                    Span::unknown(),
                )?,
                value.span(),
            )),
            _ => result,
        }
    }

    fn process_value_with_normal_error(
        &self,
        plugin: &DcmPlugin,
        value: &Value,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        match &value {
            Value::String {
                val, internal_span, ..
            } => {
                let obj = read_dcm_file(val).map_err(|e| {
                    LabeledError::new("'dcm' expects valid DICOM binary data")
                        .with_label(format!("{} [file {}]", e, val), *internal_span)
                })?;

                self.process_dicom_object(plugin, internal_span, obj, error_column)
            }
            Value::Record { val, internal_span } => {
                let record_type = get_record_string(val, "type");
                let record_name = get_record_string(val, "name");

                return match (record_type, record_name) {
                    // Probably a file Record
                    (Some("file"), Some(name)) => {
                        Err(LabeledError::new("Cannot process file records directly")
                            .with_label(
                                format!("Found file record: '{}'\n\nTo use it, extract the file name from it. Use one of:\n    dcm name\n    select name | dcm\n    get name | dcm", name),
                                *internal_span
                            ))
                    }
                    // Output generic record error
                    _ =>  Err(LabeledError::new("Cannot process records directly")
                           .with_label(
                               "Select file name or binary data from the record before passing it to dcm",
                               *internal_span
                           ))
                };
            }
            Value::Binary {
                val, internal_span, ..
            } => {
                let cursor = Cursor::new(val);
                let obj = read_dcm_stream(cursor).map_err(|e| {
                    LabeledError::new("Invalid DICOM data")
                        .with_label(e.to_string(), *internal_span)
                })?;

                self.process_dicom_object(plugin, internal_span, obj, error_column)
            }
            Value::List {
                vals,
                internal_span,
                ..
            } => {
                // Use either a dicom result or an error for each input element>
                let result: Vec<Value> = vals
                    .iter()
                    .map(|v| {
                        self.process_value(plugin, v, error_column)
                            .unwrap_or_else(|e| Value::error(e.into(), *internal_span))
                    })
                    .collect();

                Ok(Value::list(result, *internal_span))
            }
            _ => Err(LabeledError::new("Unrecognized type in stream").with_label(
                "'dcm' expects a string (filepath), binary, or column path",
                value.span(),
            )),
        }
    }

    fn process_dicom_object(
        &self,
        plugin: &DcmPlugin,
        span: &Span,
        obj: DefaultDicomObject,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        let dcm_dumper = dcm::DicomDump {
            dcm_dictionary: &plugin.dcm_dictionary,
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

        // convert index map to a record
        Ok(Value::record(Record::from_iter(index_map), *span))
    }
}

fn get_record_string<'a>(record: &'a Record, field_name: &str) -> Option<&'a str> {
    let value = record.get(field_name)?;
    let Value::String { val, .. } = value else {
        return None;
    };
    Some(val.as_str())
}
