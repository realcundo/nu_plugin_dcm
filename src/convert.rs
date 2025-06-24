use dicom::core::PrimitiveValue;
use itertools::Itertools;
use nu_protocol::{Span, Value};

#[allow(clippy::ptr_arg)]
pub fn trim_string(s: &String) -> &str {
    const TRIM_CHARS: &[char] = &[' ', '\t', '\n', '\r', '\0'];
    s.trim_matches(TRIM_CHARS)
}

pub struct Stringlike<'a>(pub &'a PrimitiveValue, pub Span);
pub struct Integerlike<'a>(pub &'a PrimitiveValue, pub Span);
pub struct Decimallike<'a>(pub &'a PrimitiveValue, pub Span);

impl From<Stringlike<'_>> for Value {
    fn from(v: Stringlike) -> Self {
        // TODO use rows/table like below?
        let val =
            v.0.to_multi_str()
                .iter()
                .map(trim_string)
                .join("\n");
        Value::string(val, v.1)
    }
}

impl From<Integerlike<'_>> for Value {
    fn from(v: Integerlike) -> Self {
        // TODO is i64 enough?
        let i =
            v.0.to_multi_int::<i64>()
                .expect("Failed to parse Integerlike to i64");

        match i.len() {
            0 => Value::nothing(v.1),
            1 => Value::int(i[0], v.1),
            _ => {
                let t: Vec<Value> = i
                    .into_iter()
                    .map(|i| Value::int(i, v.1))
                    .collect();

                // TODO use Record instead of List?
                Value::list(t, v.1)
            }
        }
    }
}

impl From<Decimallike<'_>> for Value {
    fn from(v: Decimallike) -> Self {
        // empty shortcut (not handled by to_multi_float64())
        if let PrimitiveValue::Empty = v.0 {
            return Value::nothing(v.1);
        };

        let i =
            v.0.to_multi_float64()
                .expect("Failed to parse Decimallike to f64");

        match i.len() {
            0 => Value::nothing(v.1),
            1 => Value::float(i[0], v.1),
            _ => {
                let t: Vec<Value> = i
                    .into_iter()
                    .map(|i| Value::float(i, v.1))
                    .collect();

                // TODO use Record instead of List?
                Value::list(t, v.1)
            }
        }
    }
}
