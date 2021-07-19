use dicom::{
    core::{dictionary::DictionaryEntryRef, DataDictionary, DicomValue, VR},
    object::{mem::InMemElement, InMemDicomObject},
};
use indexmap::IndexMap;
use nu_protocol::{UntaggedValue, Value};

use crate::convert::{Decimallike, Integerlike, Stringlike};

pub struct DicomDump<'a, 'b> {
    pub dictionary: &'a dyn DataDictionary<Entry = DictionaryEntryRef<'b>>,
}

impl DicomDump<'_, '_> {
    pub fn make_row_from_dicom_object(&self, obj: &InMemDicomObject) -> UntaggedValue {
        let mut d = IndexMap::with_capacity(44);

        obj.into_iter()
            .for_each(|elem| self.make_data_from_dicom_element(&mut d, elem));

        UntaggedValue::row(d)
    }

    fn make_data_from_dicom_element(&self, d: &mut IndexMap<String, Value>, elem: &InMemElement) {
        let header = elem.header();
        let vr = header.vr;

        let key = self
            .dictionary
            .by_tag(header.tag)
            .map(|r| r.alias.to_string())
            .unwrap_or_else(|| format!("{:04X},{:04X}", header.tag.group(), header.tag.element()));

        match elem.value() {
            DicomValue::Sequence { items, size: _ } => {
                let rows: Vec<Value> = items
                    .iter()
                    .map(|obj| self.make_row_from_dicom_object(obj).into())
                    .collect();

                // TODO nu doesn't require rows to have identical columns but it'd be more predictable to
                // normalise them and fill in the gaps. For now assume DCM items are identical.
                d.insert(key, UntaggedValue::Table(rows).into());
            }
            DicomValue::PixelSequence {
                offset_table: _,
                fragments: _,
            } => {
                // TODO pixel data
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
                    | VR::OB // TODO
                    | VR::OW // TODO
                    | VR::SQ // TODO
                    | VR::SV // TODO
                    | VR::UC // TODO
                    | VR::UN // TODO
                    | VR::UT => {
                        d.insert(key, Stringlike(value).into());
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
                        d.insert(key, Integerlike(value).into());
                    }
                    VR::TM | VR::DS | VR::FD | VR::FL | VR::OD | VR::OF => {
                        d.insert(key, Decimallike(value).into());
                    }
                }
            }
        }
    }
}
