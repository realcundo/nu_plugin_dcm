use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::dicomweb::{DicomWebDump, is_dicom_record};
use crate::meta::make_row_from_dicom_metadata;
use crate::reader::{read_dcm_file, read_dcm_stream};

use crate::dcm;
use dicom::object::DefaultDicomObject;
use dicom::object::StandardDataDictionary;
use indexmap::IndexMap;
use nu_plugin::{EngineInterface, Plugin, PluginCommand};
use nu_protocol::{
    Category, Example, IntoInterruptiblePipelineData, IntoPipelineData, LabeledError, PipelineData, Record, ShellError, Signals, Signature, Span,
    SyntaxShape, Value,
};

#[derive(Default, Clone)]
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

impl PluginCommand for DcmPluginCommand {
    type Plugin = DcmPlugin;

    fn name(&self) -> &str {
        "dcm"
    }

    fn description(&self) -> &str {
        "Parse DICOM object from file or binary data. Invalid DICOM objects are reported as errors and excluded from the output."
    }

    fn signature(&self) -> Signature {
        /*
        // Some example DICOM fields
        let dicom_record_fields = vec![
            ("StudyInstanceUID".to_string(), Type::String),
            ("StudyDate".to_string(), Type::String),
            ("Modality".to_string(), Type::Float),
            (
                "PixelSpacing".to_string(),
                Type::List(Box::new(Type::Float)),
            ),
            ("...".to_string(), Type::Any),
        ];

        let file_record_fields = vec![
            ("name".to_string(), Type::String),
            ("type".to_string(), Type::String),
        ];

        let dicom_record_type = Type::Record(dicom_record_fields.into_boxed_slice());
        let file_record_type = Type::Record(file_record_fields.into_boxed_slice());
        */

        Signature::build(nu_plugin::PluginCommand::name(self))
            // TODO is it possible to specify all input/output types? The plugin is very
            // dynamic and e.g. `echo file.dcm | wrap foo | dcm foo` failed the type check...
            /*
            .input_output_types(
                vec![
                    // String (filename) -> Record (DICOM data)
                    (Type::String, dicom_record_type.clone()),
                    // Binary (DICOM data) -> Record (DICOM data)
                    (Type::Binary, dicom_record_type.clone()),
                    // Record (file data) -> Record (DICOM data)
                    (file_record_type.clone(), dicom_record_type.clone()),
                    // List of Strings (filenames) -> List of Records
                    (Type::List(Box::new(Type::String)), Type::List(Box::new(dicom_record_type.clone()))),
                    // List of Binary -> List of Records (e.g. [(open 1.dcm), (open 2.dcm)] | dcm)
                    (Type::List(Box::new(Type::Binary)), Type::List(Box::new(dicom_record_type.clone()))),
                    // List of file records) -> List of Records (e.g. `ls *.dcm | dcm name`)
                    (Type::List(Box::new(file_record_type.clone())), Type::List(Box::new(dicom_record_type))),
                ])
              */
            .named(
                "error",
                SyntaxShape::String,
                "If an error occurs when Dicom object is parsed, the error message will be inserted in this column instead producing an error result.",
                Some('e'))
            .category(Category::Formats)  // More appropriate category
            .search_terms(vec!["dicom".to_string(), "medical".to_string(), "parse".to_string()])
            .description("Parse DICOM files and binary data")
            .extra_description("Parse DICOM objects from files or binary data. Invalid DICOM objects are reported as errors and excluded from the output unless --error flag is used.")
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Parse a DICOM file by passing binary data",
                example: "open file.dcm | dcm",
                result: Some(Value::test_record(Record::from_iter([
                    ("PatientName".to_string(), Value::test_string("John Doe")),
                    ("Modality".to_string(), Value::test_string("CT")),
                    ("StudyDate".to_string(), Value::test_string("20231201")),
                    ("ImageType".to_string(), Value::test_string("ORIGINAL\\PRIMARY")),
                ]))),
            },
            Example { description: "Parse DICOM files from a list", example: "ls *.dcm | dcm", result: None },
            Example { description: "Parse a specific file by filename", example: "\"file.dcm\" | dcm", result: None },
            Example { description: "Parse with error handling", example: "ls *.dcm | dcm --error parse_error", result: None },
        ]
    }

    fn run(
        &self,
        plugin: &DcmPlugin,
        engine: &EngineInterface,
        call: &nu_plugin::EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let current_dir = engine
            .get_current_dir()
            .map(PathBuf::from);

        let input_span = input
            .span()
            .unwrap_or(call.head);
        let input_metadata = input.metadata();

        let error_column = call.get_flag::<String>("error")?;

        // run
        // TODO find a way without cloning? ListStream::map() requires 'static lifetime, maybe I can use iterators directly?
        let output = self.process_pipeline_data(plugin.clone(), current_dir, error_column, &input_span, input)?;

        // Forward DataSource metadata from input to output, but clear any content type. This keeps the source.
        let output_metadata = input_metadata.map(|m| m.with_content_type(None));

        Ok(output.set_metadata(output_metadata))
    }
}

impl DcmPluginCommand {
    pub fn process_pipeline_data(
        &self,
        plugin: DcmPlugin,
        current_dir: Result<PathBuf, ShellError>,
        error_column: Option<String>,
        input_span: &Span,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        match input {
            // no-op
            PipelineData::Empty => Ok(PipelineData::Empty),

            // process value directly
            PipelineData::Value(value, ..) => {
                Self::process_value(&plugin, current_dir.as_deref(), &value, &error_column).map(Value::into_pipeline_data)
            }

            // map list of values one by one
            PipelineData::ListStream(list_stream, ..) => {
                // TODO should this fail immediately or generate errors?
                let mapped_stream = list_stream.map(move |v| match Self::process_value(&plugin, current_dir.as_deref(), &v, &error_column) {
                    Ok(value) => value,
                    Err(e) => Value::error(e.into(), v.span()),
                });

                Ok(mapped_stream.into_pipeline_data(*input_span, Signals::EMPTY))
            }

            // process input bytestream directly without collecting it into memory
            PipelineData::ByteStream(byte_stream, ..) => {
                let byte_stream_reader = byte_stream
                    .reader()
                    .ok_or_else(|| LabeledError::new("Empty bytestream"))?;

                let obj = read_dcm_stream(byte_stream_reader)
                    .map_err(|e| LabeledError::new("Invalid DICOM data").with_label(e.to_string(), *input_span))?;

                Self::process_dicom_object(&plugin, input_span, obj, &error_column).map(Value::into_pipeline_data)
            }
        }
    }

    fn process_value(
        plugin: &DcmPlugin,
        current_dir: Result<&Path, &ShellError>,
        value: &Value,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        let result = Self::process_value_with_normal_error(plugin, current_dir, value, error_column);

        // TODO better value.span().unwrap()
        match (error_column, &result) {
            (Some(error_column), Err(err)) => Ok(Value::record(
                Record::from_raw_cols_vals(
                    vec![error_column.to_string()],
                    vec![Value::string(
                        err.msg
                            .to_string(),
                        value.span(),
                    )],
                    Span::unknown(),
                    Span::unknown(),
                )?,
                value.span(),
            )),
            _ => result,
        }
    }

    fn process_value_with_normal_error(
        plugin: &DcmPlugin,
        current_dir: Result<&Path, &ShellError>,
        value: &Value,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        match &value {
            Value::String { val, internal_span, .. } => {
                // make absolute if needed
                let file = resolve_path(val, current_dir, value.span())?;

                // TODO add some heuristics to determine if the input string is filename or DICOM binary data connverted to utf-8 by nu?
                // (see `ByteStream::into_value()` which does the conversion to string.)

                let obj = read_dcm_file(&file).map_err(|e| {
                    // Report a better error if the input string looks like DICOM binary data with preamble.
                    // TODO this is messy. In fact the whole error reporting is messy.
                    let text = if val.get(128..132) == Some("DICM") {
                        "Input string looks like DICOM binary data. Either pass binary data, or a filename.".to_string()
                    } else {
                        format!("{} [file {}]", e, file.to_string_lossy())
                    };

                    LabeledError::new("`dcm` expects valid DICOM binary data").with_label(text, *internal_span)
                })?;

                Self::process_dicom_object(plugin, internal_span, obj, error_column)
            }
            Value::Record { val, internal_span } => {
                // Check if a file record
                let record_type = get_record_string(val, "type");
                let record_name = get_record_string(val, "name");

                if let (Some(record_type), Some(record_name)) = (record_type, record_name) {
                    if record_type == "file" {
                        // make absolute if needed
                        let file = resolve_path(record_name, current_dir, value.span())?;

                        // merge with file-reading above (Value::String)?
                        let obj = read_dcm_file(&file).map_err(|e| {
                            let text = format!("{} [file {}]", e, file.to_string_lossy());

                            LabeledError::new("`dcm` expects valid DICOM binary data").with_label(text, *internal_span)
                        })?;

                        return Self::process_dicom_object(plugin, internal_span, obj, error_column);
                    }
                }

                // Check if it looks like a dicomweb record.
                if record_name.is_none() && record_type.is_none() && is_dicom_record(val) {
                    let dcm_dumper = DicomWebDump::with_dictionary(&plugin.dcm_dictionary);
                    let result = dcm_dumper
                        .process_dicomweb_record(val, *internal_span)
                        .map_err(|e| LabeledError::new("Failed to proess DicomWeb record").with_label(e.to_string(), e.span()))?;

                    return Ok(result);
                }

                // Output generic error
                Err(LabeledError::new("Cannot process records directly, unless they are Fila or DicomWeb records")
                    .with_label("For files, select file name, binary data, or use records with `name` and `type`", *internal_span))
            }
            Value::Binary { val, internal_span, .. } => {
                let cursor = Cursor::new(val);
                let obj = read_dcm_stream(cursor).map_err(|e| LabeledError::new("Invalid DICOM data").with_label(e.to_string(), *internal_span))?;

                Self::process_dicom_object(plugin, internal_span, obj, error_column)
            }
            Value::List { vals, internal_span, .. } => {
                // Use either a dicom result or an error for each input element>
                let result: Vec<Value> = vals
                    .iter()
                    .map(|v| Self::process_value(plugin, current_dir, v, error_column).unwrap_or_else(|e| Value::error(e.into(), *internal_span)))
                    .collect();

                Ok(Value::list(result, *internal_span))
            }
            _ => Err(LabeledError::new("Unrecognized type in stream")
                .with_label("'dcm' expects a string (filepath), binary, or column path", value.span())),
        }
    }

    fn process_dicom_object(
        plugin: &DcmPlugin,
        span: &Span,
        obj: DefaultDicomObject,
        error_column: &Option<String>,
    ) -> Result<Value, LabeledError> {
        let dcm_dumper = dcm::DicomDump { dcm_dictionary: &plugin.dcm_dictionary };

        let mut index_map = IndexMap::with_capacity(1000);

        // make sure that when --error is used, the column always exists
        if let Some(error_column) = error_column {
            index_map.insert(error_column.to_string(), Value::string(String::new(), *span));
        }

        // dump both metadata and data into a single table
        make_row_from_dicom_metadata(span, &mut index_map, obj.meta());
        dcm_dumper.make_row_from_dicom_object(span, &mut index_map, &obj);

        // convert index map to a record
        Ok(Value::record(Record::from_iter(index_map), *span))
    }
}

fn get_record_string<'a>(
    record: &'a Record,
    field_name: &str,
) -> Option<&'a str> {
    let value = record.get(field_name)?;
    let Value::String { val, .. } = value else {
        return None;
    };
    Some(val.as_str())
}

fn resolve_path(
    filename: &str,
    current_dir: Result<&Path, &ShellError>,
    span: Span,
) -> Result<PathBuf, LabeledError> {
    use std::path::Path;

    let path = Path::new(filename);

    // If path is already absolute, return it as-is
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    // Path is relative, need to resolve against current working directory
    let current_dir = current_dir.map_err(|e| {
        LabeledError::new("Failed to get current working directory")
            .with_label(format!("Cannot resolve relative path '{filename}'\n\nError: {e}"), span)
    })?;

    Ok(PathBuf::from(current_dir).join(filename))
}
