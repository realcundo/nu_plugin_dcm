use std::str::FromStr;

use bigdecimal::BigDecimal;
use dicom::core::PrimitiveValue;
use itertools::Itertools;
use nu_protocol::{Primitive, UntaggedValue};

pub fn trim_string(s: &String) -> &str {
    const TRIM_CHARS: &[char] = &[' ', '\t', '\n', '\r', '\0'];
    s.trim_matches(TRIM_CHARS)
}

pub struct Stringlike<'a>(pub &'a PrimitiveValue);
pub struct Integerlike<'a>(pub &'a PrimitiveValue);
pub struct Decimallike<'a>(pub &'a PrimitiveValue);

impl From<Stringlike<'_>> for nu_protocol::Value {
    fn from(v: Stringlike) -> Self {
        // TODO use rows/table like below?
        let s = v.0.to_multi_str().iter().map(trim_string).join("\n");
        UntaggedValue::Primitive(Primitive::String(s)).into()
    }
}

impl From<Integerlike<'_>> for nu_protocol::Value {
    fn from(v: Integerlike) -> Self {
        let i = v.0.to_multi_int::<i128>().unwrap();

        match i.len() {
            0 => UntaggedValue::Primitive(Primitive::Nothing).into(),
            1 => UntaggedValue::Primitive(Primitive::BigInt(i[0].into())).into(),
            _ => {
                let t: Vec<nu_protocol::Value> = i
                    .into_iter()
                    .map(|i| UntaggedValue::Primitive(Primitive::BigInt(i.into())).into())
                    .collect();

                // TODO use Row instead of full table?
                nu_protocol::UntaggedValue::Table(t).into()
            }
        }
    }
}

impl From<Decimallike<'_>> for nu_protocol::Value {
    fn from(v: Decimallike) -> Self {
        let s = v.0.to_multi_str();
        let s = s.iter().map(trim_string).collect::<Vec<_>>();

        match s.len() {
            0 => UntaggedValue::Primitive(Primitive::Nothing).into(),
            1 => UntaggedValue::Primitive(Primitive::Decimal(BigDecimal::from_str(s[0]).unwrap()))
                .into(),
            _ => {
                let t: Vec<nu_protocol::Value> = s
                    .into_iter()
                    .map(|s| {
                        UntaggedValue::Primitive(Primitive::Decimal(
                            BigDecimal::from_str(s).unwrap(),
                        ))
                        .into()
                    })
                    .collect();

                // TODO use Row instead of full table?
                nu_protocol::UntaggedValue::Table(t).into()
            }
        }
    }
}
