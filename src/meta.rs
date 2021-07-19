use dicom::object::FileMetaTable;
use indexmap::IndexMap;
use nu_protocol::UntaggedValue;

use crate::convert::trim_string;

pub fn make_row_from_dicom_metadata(meta: &FileMetaTable) -> UntaggedValue {
    let mut d = IndexMap::with_capacity(10);

    // TODO add more metadata

    d.insert(
        "Transfer Syntax".to_string(),
        UntaggedValue::string(trim_string(&meta.transfer_syntax)).into(),
    );

    d.insert(
        "Media Storage SOP Class UID".to_string(),
        UntaggedValue::string(trim_string(&meta.media_storage_sop_class_uid)).into(),
    );

    d.insert(
        "Media Storage SOP Instance UID".to_string(),
        UntaggedValue::string(trim_string(&meta.media_storage_sop_instance_uid)).into(),
    );

    UntaggedValue::row(d)
}
