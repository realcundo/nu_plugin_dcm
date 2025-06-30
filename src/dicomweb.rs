use std::str::FromStr;

use dicom::core::{DataDictionary, VR};
use nu_protocol::{Record, Span, Value};
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum DicomWebError {
    #[snafu(display("Missing required column: {column}"))]
    MissingRequiredColumn { column: &'static str, span: Span },
    #[snafu(display("Unexpected type: expected {expected}, got {actual}"))]
    InvalidType { expected: &'static str, actual: nu_protocol::Type, span: Span },
    #[snafu(display("Unexpected value: expected {expected}, got {actual}"))]
    InvalidValue { expected: &'static str, actual: String, span: Span },
}

impl DicomWebError {
    pub fn span(&self) -> Span {
        match self {
            DicomWebError::MissingRequiredColumn { span, .. } => *span,
            DicomWebError::InvalidType { span, .. } => *span,
            DicomWebError::InvalidValue { span, .. } => *span,
        }
    }
}

/// Checks if a record is a DICOM record by looking at the first 50 fields.
pub fn is_dicom_record(record: &Record) -> bool {
    record
        .iter()
        .take(50) // only check first 50 tags. Or should it check all? But then we can just as well try to parse...
        .all(|(potential_tag, potential_value)| {
            // Check https://dicom.nema.org/medical/dicom/current/output/chtml/part18/sect_F.2.2.html
            if let Ok(potential_record) = potential_value.as_record() {
                potential_tag.len() == 8
                    // the tag must be a valid hex string
                    && potential_tag
                        .chars()
                        .all(|c| c.is_ascii_hexdigit())
                    // must contain "vr"
                    && potential_record.contains("vr")
                    // tags must be one of the following
                    && (potential_record
                        .iter()
                        .all(|(key, _value)| key == "Value" || key == "BulkDataURI" || key == "InlineBinary" || key == "vr"))
            } else {
                false
            }
        })
}

pub struct DicomWebDump<'a, 'd> {
    dcm_dictionary: &'a dyn DataDictionary<Entry = dicom::core::dictionary::DataDictionaryEntryRef<'d>>,
}

impl<'a, 'd> DicomWebDump<'a, 'd>
where
    'd: 'a,
{
    pub fn with_dictionary(dcm_dictionary: &'a dyn DataDictionary<Entry = dicom::core::dictionary::DataDictionaryEntryRef<'d>>) -> Self {
        Self { dcm_dictionary }
    }
}

impl DicomWebDump<'_, '_> {
    pub fn process_dicomweb_list(
        &self,
        list: &[Value],
        list_span: Span,
    ) -> Result<Value, DicomWebError> {
        let remapped = list
            .iter()
            .map(|value| {
                let record = value
                    .as_record()
                    .map_err(|_| DicomWebError::InvalidType { expected: "record (in a list)", actual: value.get_type(), span: value.span() })?;

                self.process_dicomweb_record(record, value.span())
            })
            .collect::<Result<Vec<Value>, DicomWebError>>()?;

        Ok(Value::list(remapped, list_span))
    }

    pub fn process_dicomweb_record(
        &self,
        record: &Record,
        record_span: Span,
    ) -> Result<Value, DicomWebError> {
        // iterate over all keys and expect to contain nested record with "vr" and "Value" keys
        let remapped = record
            .iter()
            .map(|(tag, value)| {
                let tag = match dicom::core::Tag::from_str(tag) {
                    Ok(tag) => tag,
                    Err(_) => {
                        return Err(DicomWebError::InvalidValue {
                            expected: "a hexadecimal string representing a DICOM tag",
                            actual: tag.to_string(),
                            span: record_span,
                        });
                    }
                };

                let key = self
                    .dcm_dictionary
                    .by_tag(tag)
                    .map(|r| {
                        r.alias
                            .to_string()
                    })
                    .unwrap_or_else(|| format!("{:04X},{:04X}", tag.group(), tag.element()));

                let converted_value = self.convert_value(value)?;
                Ok((key, converted_value))
            })
            .collect::<Result<Vec<(String, Value)>, DicomWebError>>()?;

        Ok(Value::record(Record::from_iter(remapped), record_span))
    }

    fn convert_value(
        &self,
        value: &Value,
    ) -> Result<Value, DicomWebError> {
        let record_span = value.span();

        // check if the value is a record
        let record = value
            .as_record()
            .map_err(|_| DicomWebError::InvalidType { expected: "record with VR and Value", actual: value.get_type(), span: record_span })?;

        // value must have "vr"
        let vr = record
            .get("vr")
            .ok_or(DicomWebError::MissingRequiredColumn { column: "vr", span: record_span })?;

        let vr_span = vr.span();
        let vr = vr
            .as_str()
            .map_err(|_| DicomWebError::InvalidType { expected: "string representing a VR", actual: vr.get_type(), span: vr_span })?;

        // TODO needed to parse?
        let vr =
            dicom::core::VR::from_str(vr).map_err(|_| DicomWebError::InvalidValue { expected: "valid VR", actual: vr.to_string(), span: vr_span })?;

        // only fetch value. ignore `BulkDataURI`/`InlineBinary`
        let value = match record.get("Value") {
            Some(value) => value,
            None => {
                // don't return error if BulkDataURI/InlineBinary exist or if no other fields are present (just having "vr" with nothing else is valid)
                if record.len() == 1 || record.contains("BulkDataURI") || record.contains("InlineBinary") {
                    return Ok(Value::nothing(record_span));
                } else {
                    return Err(DicomWebError::MissingRequiredColumn { column: "Value|BulkDataURI|InlineBinary", span: record_span });
                }
            }
        };

        match vr {
            VR::SQ => {
                // expect Value to be a list and reurse for each item, expecting it to be a record
                let list = value
                    .as_list()
                    .map_err(|_| DicomWebError::InvalidType { expected: "list", actual: value.get_type(), span: value.span() })?;

                self.process_dicomweb_list(list, value.span())
            }

            // String-like
            VR::AE | VR::AS | VR::CS | VR::DA | VR::DT | VR::LO | VR::LT | VR::SH | VR::ST | VR::TM | VR::UC | VR::UI | VR::UR | VR::UT => {
                self.convert_stringlike_value(value)
            }

            VR::PN => self.convert_pn_value(value),

            // Integer-like
            VR::IS | VR::SS | VR::US | VR::SL | VR::UL | VR::OL | VR::OV | VR::SV | VR::UV => self.convert_integer_like_value(value),

            // Decimal-like
            VR::DS | VR::FL | VR::FD | VR::OF | VR::OD => self.convert_decimal_like_value(value),

            // Not yet handled
            VR::AT => todo!(),
            VR::OB => todo!(),
            VR::OW => todo!(),
            VR::UN => todo!(),
        }
    }

    fn convert_integer_like_value(
        &self,
        value: &Value,
    ) -> Result<Value, DicomWebError> {
        match value {
            Value::Nothing { .. } => Ok(value.clone()),
            Value::Int { .. } => Ok(value.clone()),
            Value::List { vals, .. } => {
                if vals.is_empty() {
                    return Ok(Value::nothing(value.span()));
                }

                let int_results: Result<Vec<Value>, DicomWebError> = vals
                    .iter()
                    .map(|v| match v {
                        Value::Int { .. } => Ok(v.clone()),
                        _ => Err(DicomWebError::InvalidType { expected: "list of integers", actual: v.get_type(), span: v.span() }),
                    })
                    .collect();

                let mut collected_vals = int_results?;

                if collected_vals.len() == 1 {
                    Ok(collected_vals.remove(0))
                } else {
                    Ok(Value::list(collected_vals, value.span()))
                }
            }
            _ => Err(DicomWebError::InvalidType { expected: "integer or a list of integers", actual: value.get_type(), span: value.span() }),
        }
    }

    fn convert_decimal_like_value(
        &self,
        value: &Value,
    ) -> Result<Value, DicomWebError> {
        match value {
            Value::Nothing { .. } => Ok(value.clone()),
            Value::Float { .. } => Ok(value.clone()),
            Value::Int { val, internal_span } => Ok(Value::float(*val as f64, *internal_span)),
            Value::List { vals, .. } => {
                if vals.is_empty() {
                    return Ok(Value::nothing(value.span()));
                }

                let float_results: Result<Vec<Value>, DicomWebError> = vals
                    .iter()
                    .map(|v| match v {
                        Value::Float { .. } => Ok(v.clone()),
                        Value::Int { val, internal_span } => Ok(Value::float(*val as f64, *internal_span)),
                        _ => Err(DicomWebError::InvalidType { expected: "list of numbers", actual: v.get_type(), span: v.span() }),
                    })
                    .collect();

                let mut collected_vals = float_results?;

                if collected_vals.len() == 1 {
                    Ok(collected_vals.remove(0))
                } else {
                    Ok(Value::list(collected_vals, value.span()))
                }
            }
            _ => Err(DicomWebError::InvalidType { expected: "number or a list of number", actual: value.get_type(), span: value.span() }),
        }
    }

    fn convert_pn_value(
        &self,
        value: &Value,
    ) -> Result<Value, DicomWebError> {
        if value.is_nothing() {
            return Ok(value.clone());
        }

        // expect a list of records with an "Alphabetic" key
        let list = value
            .as_list()
            .map_err(|_| DicomWebError::InvalidType { expected: "list of PatientName records", actual: value.get_type(), span: value.span() })?;

        if list.is_empty() {
            return Ok(Value::nothing(value.span()));
        }

        let remapped: Result<Vec<Value>, DicomWebError> = list
            .iter()
            .map(|v| {
                let record = v
                    .as_record()
                    .map_err(|_| DicomWebError::InvalidType { expected: "PatientName record", actual: v.get_type(), span: v.span() })?;

                // Prioritise "Alphabetic" field, followed by "Ideographic" and "Phonetic"
                let pn_value = record
                    .get("Alphabetic")
                    .or_else(|| record.get("Ideographic"))
                    .or_else(|| record.get("Phonetic"))
                    .ok_or_else(|| DicomWebError::MissingRequiredColumn { column: "Alphabetic|Ideographic|Phonetic", span: v.span() })?;

                Ok(pn_value.clone())
            })
            .collect();

        let mut collected_vals = remapped?;

        if collected_vals.len() == 1 {
            Ok(collected_vals.remove(0))
        } else {
            Ok(Value::list(collected_vals, value.span()))
        }
    }

    fn convert_stringlike_value(
        &self,
        value: &Value,
    ) -> Result<Value, DicomWebError> {
        match value {
            Value::Nothing { .. } => Ok(value.clone()),
            Value::String { .. } => Ok(value.clone()),
            Value::List { vals, .. } => {
                if vals.is_empty() {
                    return Ok(Value::nothing(value.span()));
                }

                let string_results: Result<Vec<Value>, DicomWebError> = vals
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .map_err(|_| DicomWebError::InvalidType { expected: "string", actual: v.get_type(), span: v.span() })?;

                        Ok(v.clone())
                    })
                    .collect();

                let collected_vals = string_results?;

                if collected_vals.len() == 1 {
                    Ok(collected_vals
                        .into_iter()
                        .next()
                        .unwrap())
                } else {
                    Ok(Value::list(collected_vals, value.span()))
                }
            }
            Value::Record { val, .. } if val.is_empty() => Ok(Value::nothing(value.span())),
            _ => {
                Err(DicomWebError::InvalidType { expected: "string, list of strings or empty record", actual: value.get_type(), span: value.span() })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dicom::dictionary_std::StandardDataDictionary;
    use test_case::test_case;

    fn get_dicom_web_dump() -> DicomWebDump<'static, 'static> {
        DicomWebDump::with_dictionary(&StandardDataDictionary)
    }

    #[test_case(
        Value::test_nothing(),
        Value::test_nothing(); "nothing value")]
    #[test_case(
        Value::test_string("test"),
        Value::test_string("test"); "string value")]
    #[test_case(
        Value::test_list(vec![Value::test_string("test1"), Value::test_string("test2")]),
        Value::test_list(vec![Value::test_string("test1"), Value::test_string("test2")]); "list of strings")]
    #[test_case(
        Value::test_list(vec![]),
        Value::nothing(Span::test_data()); "empty list")]
    #[test_case(
        Value::test_record(Record::new()),
        Value::nothing(Span::test_data()); "empty record")]
    fn test_convert_stringlike_value_success(
        input: Value,
        expected: Value,
    ) -> Result<(), DicomWebError> {
        let result = get_dicom_web_dump().convert_stringlike_value(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Value::test_int(1); "integer value")]
    #[test_case(Value::test_float(1.0); "float value")]
    #[test_case(Value::test_bool(true); "boolean value")]
    #[test_case(Value::test_list(vec![Value::test_int(1)]); "list of integers")]
    fn test_convert_stringlike_value_error(input: Value) {
        let result = get_dicom_web_dump().convert_stringlike_value(&input);
        assert!(result.is_err());
    }

    #[test_case(
        Value::test_nothing(),
        Value::test_nothing(); "nothing value")]
    #[test_case(
        Value::test_int(1),
        Value::test_int(1); "integer value")]
    #[test_case(
        Value::test_list(vec![Value::test_int(1), Value::test_int(2)]),
        Value::test_list(vec![Value::test_int(1), Value::test_int(2)]); "list of integers")]
    #[test_case(
        Value::test_list(vec![]),
        Value::nothing(Span::test_data()); "empty list")]
    fn test_convert_integer_like_value_success(
        input: Value,
        expected: Value,
    ) -> Result<(), DicomWebError> {
        let result = get_dicom_web_dump().convert_integer_like_value(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Value::test_string("abc"); "string value")]
    #[test_case(Value::test_float(1.0); "float value")]
    #[test_case(Value::test_bool(true); "boolean value")]
    #[test_case(Value::test_list(vec![Value::test_string("abc")]); "list of strings")]
    fn test_convert_integer_like_value_error(input: Value) {
        let result = get_dicom_web_dump().convert_integer_like_value(&input);
        assert!(result.is_err());
    }

    #[test_case(
        Value::test_nothing(),
        Value::test_nothing(); "nothing value")]
    #[test_case(
        Value::test_float(1.0),
        Value::test_float(1.0); "float value")]
    #[test_case(
        Value::test_int(1),
        Value::test_float(1.0); "integer value")]
    #[test_case(
        Value::test_list(vec![Value::test_float(1.0), Value::test_float(2.0)]),
        Value::test_list(vec![Value::test_float(1.0), Value::test_float(2.0)]); "list of floats")]
    #[test_case(
        Value::test_list(vec![Value::test_int(1), Value::test_int(2)]),
        Value::test_list(vec![Value::test_float(1.0), Value::test_float(2.0)]); "list of integers to floats")]
    #[test_case(
        Value::test_list(vec![]),
        Value::nothing(Span::test_data()); "empty list")]
    fn test_convert_decimal_like_value_success(
        input: Value,
        expected: Value,
    ) -> Result<(), DicomWebError> {
        let result = get_dicom_web_dump().convert_decimal_like_value(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Value::test_string("abc"); "string value")]
    #[test_case(Value::test_bool(true); "boolean value")]
    #[test_case(Value::test_list(vec![Value::test_string("abc")]); "list of strings")]
    fn test_convert_decimal_like_value_error(input: Value) {
        let result = get_dicom_web_dump().convert_decimal_like_value(&input);
        assert!(result.is_err());
    }

    #[test_case(
        Value::test_nothing(),
        Value::test_nothing(); "nothing value")]
    #[test_case(
        Value::test_list(vec![Value::test_record(Record::from_iter(vec![("Alphabetic".to_string(), Value::test_string("Doe^John"))]))]),
        Value::test_string("Doe^John"); "single PN value")]
    #[test_case(
        Value::test_list(vec![
            Value::test_record(Record::from_iter(vec![("Alphabetic".to_string(), Value::test_string("Doe^John"))])),
            Value::test_record(Record::from_iter(vec![("Alphabetic".to_string(), Value::test_string("Smith^Jane"))])),
        ]),
        Value::test_list(vec![Value::test_string("Doe^John"), Value::test_string("Smith^Jane")]); "multiple PN values")]
    #[test_case(
        Value::test_list(vec![Value::test_record(Record::from_iter(vec![("Ideographic".to_string(), Value::test_string("Doe=John"))]))]),
        Value::test_string("Doe=John"); "single PN value with Ideographic")]
    #[test_case(
        Value::test_list(vec![Value::test_record(Record::from_iter(vec![("Phonetic".to_string(), Value::test_string("Doe^John"))]))]),
        Value::test_string("Doe^John"); "single PN value with Phonetic")]
    #[test_case(
        Value::test_list(vec![
            Value::test_record(Record::from_iter(vec![("Ideographic".to_string(), Value::test_string("Doe=John"))])),
            Value::test_record(Record::from_iter(vec![("Alphabetic".to_string(), Value::test_string("Smith^Jane"))])),
        ]),
        Value::test_list(vec![Value::test_string("Doe=John"), Value::test_string("Smith^Jane")]); "multiple PN values with mixed types")]
    #[test_case(
        Value::test_list(vec![]),
        Value::nothing(Span::test_data()); "empty list")]
    fn test_convert_pn_value_success(
        input: Value,
        expected: Value,
    ) -> Result<(), DicomWebError> {
        let result = get_dicom_web_dump().convert_pn_value(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Value::test_string("abc"); "string value")]
    #[test_case(Value::test_int(1); "integer value")]
    #[test_case(Value::test_bool(true); "boolean value")]
    #[test_case(Value::test_list(vec![Value::test_record(Record::new())]); "list with missing PN keys")]
    fn test_convert_pn_value_error(input: Value) {
        let result = get_dicom_web_dump().convert_pn_value(&input);
        assert!(result.is_err());
    }
}
