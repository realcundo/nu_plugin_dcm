use dicom::{
    core::{dictionary::DictionaryEntryRef, DataDictionary, DicomValue, VR},
    object::{mem::InMemElement, InMemDicomObject},
};
use indexmap::IndexMap;
use nu_protocol::{Span, Spanned, Value};

use crate::convert::{Decimallike, Integerlike, Stringlike};

pub struct DicomDump<'a, 'b> {
    pub dcm_dictionary: &'a dyn DataDictionary<Entry = DictionaryEntryRef<'b>>,
}

impl DicomDump<'_, '_> {
    pub fn make_row_from_dicom_object(
        &self,
        span: &Span,
        index_map: &mut IndexMap<String, Value>,
        obj: &InMemDicomObject,
    ) {
        obj.into_iter()
            .for_each(|elem| self.make_data_from_dicom_element(span, index_map, elem));
    }

    fn make_data_from_dicom_element(
        &self,
        span: &Span,
        index_map: &mut IndexMap<String, Value>,
        elem: &InMemElement,
    ) {
        let header = elem.header();
        let vr = header.vr;

        let key = self
            .dcm_dictionary
            .by_tag(header.tag)
            .map(|r| r.alias.to_string())
            .unwrap_or_else(|| format!("{:04X},{:04X}", header.tag.group(), header.tag.element()));

        match elem.value() {
            DicomValue::Sequence { items, size: _ } => {
                let rows: Vec<Value> = items
                    .iter()
                    .map(|obj| {
                        let mut nested_index_map = IndexMap::with_capacity(1000);
                        self.make_row_from_dicom_object(span, &mut nested_index_map, obj);

                        let nested_index_map = Spanned {
                            item: nested_index_map,
                            span: *span,
                        };

                        Value::from(nested_index_map)
                    })
                    .collect();

                // TODO nu doesn't require rows to have identical columns but it'd be more predictable to
                // normalise them and fill in the gaps. For now assume DCM items are identical.
                let table = Value::List {
                    vals: rows,
                    span: *span,
                };

                index_map.insert(key, table);
            }
            DicomValue::PixelSequence {
                offset_table: _,
                fragments: _,
            } => {
                // no-op, pixel data are not read
            }
            DicomValue::Primitive(value) => {
                match vr {
                    VR::CS
                    | VR::UI
                    | VR::SH
                    | VR::LO
                    | VR::DT // TODO parse DT into nu datetime
                    | VR::PN // TODO parse PN?
                    | VR::AE
                    | VR::LT
                    | VR::ST
                    | VR::UR
                    | VR::AS // TODO
                    | VR::AT // TODO
                    | VR::OB // TODO pixel data are never read
                    | VR::OW // TODO pixel data are never read
                    | VR::SQ // TODO
                    | VR::SV // TODO
                    | VR::UC // TODO
                    | VR::UN // TODO
                    | VR::UT => {
                        index_map.insert(key, Stringlike(value, *span).into());
                    }
                    VR::DA
                    | VR::IS
                    | VR::US
                    | VR::SS
                    | VR::OL
                    | VR::OV
                    | VR::SL
                    | VR::UL
                    | VR::UV => {
                        index_map.insert(key, Integerlike(value, *span).into());
                    }
                    VR::TM | VR::DS | VR::FD | VR::FL | VR::OD | VR::OF => {
                        index_map.insert(key, Decimallike(value, *span).into());
                    }
                }
            }
        }
    }
}
