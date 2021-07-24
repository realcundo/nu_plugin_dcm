use dicom::object::FileMetaTable;
use indexmap::IndexMap;
use nu_protocol::{UntaggedValue, Value};

use crate::convert::trim_string;

pub fn make_row_from_dicom_metadata(index_map: &mut IndexMap<String, Value>, meta: &FileMetaTable) {
    // TODO add more metadata

    index_map.insert(
        "TransferSyntax".to_string(),
        UntaggedValue::string(trim_string(&meta.transfer_syntax)).into(),
    );

    index_map.insert(
        "MediaStorageSOPClassUID".to_string(),
        UntaggedValue::string(trim_string(&meta.media_storage_sop_class_uid)).into(),
    );

    index_map.insert(
        "MediaStorageSOPInstanceUID".to_string(),
        UntaggedValue::string(trim_string(&meta.media_storage_sop_instance_uid)).into(),
    );
}
