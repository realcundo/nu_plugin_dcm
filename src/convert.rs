use dicom::core::PrimitiveValue;
use itertools::Itertools;
use nu_protocol::Span;

#[allow(clippy::ptr_arg)]
pub fn trim_string(s: &String) -> &str {
    const TRIM_CHARS: &[char] = &[' ', '\t', '\n', '\r', '\0'];
    s.trim_matches(TRIM_CHARS)
}

pub struct Stringlike<'a>(pub &'a PrimitiveValue, pub Span);
pub struct Integerlike<'a>(pub &'a PrimitiveValue, pub Span);
pub struct Decimallike<'a>(pub &'a PrimitiveValue, pub Span);

impl From<Stringlike<'_>> for nu_protocol::Value {
    fn from(v: Stringlike) -> Self {
        // TODO use rows/table like below?
        let val = v.0.to_multi_str().iter().map(trim_string).join("\n");
        nu_protocol::Value::String { val, span: v.1 }
    }
}

impl From<Integerlike<'_>> for nu_protocol::Value {
    fn from(v: Integerlike) -> Self {
        // TODO is i64 enough?
        let i =
            v.0.to_multi_int::<i64>()
                .expect("Failed to parse Integerlike to i64");

        match i.len() {
            0 => nu_protocol::Value::Nothing { span: v.1 },
            1 => nu_protocol::Value::Int {
                val: i[0],
                span: v.1,
            },
            _ => {
                let t: Vec<nu_protocol::Value> = i
                    .into_iter()
                    .map(|i| nu_protocol::Value::Int { val: i, span: v.1 })
                    .collect();

                // TODO use Record instead of List?
                nu_protocol::Value::List { vals: t, span: v.1 }
            }
        }
    }
}

impl From<Decimallike<'_>> for nu_protocol::Value {
    fn from(v: Decimallike) -> Self {
        // empty shortcut (not handled by to_multi_float64())
        if let PrimitiveValue::Empty = v.0 {
            return nu_protocol::Value::Nothing { span: v.1 };
        };

        let i =
            v.0.to_multi_float64()
                .expect("Failed to parse Decimallike to f64");

        match i.len() {
            0 => nu_protocol::Value::Nothing { span: v.1 },
            1 => nu_protocol::Value::Float {
                val: i[0],
                span: v.1,
            },
            _ => {
                let t: Vec<nu_protocol::Value> = i
                    .into_iter()
                    .map(|i| nu_protocol::Value::Float { val: i, span: v.1 })
                    .collect();

                // TODO use Record instead of List?
                nu_protocol::Value::List { vals: t, span: v.1 }
            }
        }
    }
}
