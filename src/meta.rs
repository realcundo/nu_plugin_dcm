use dicom::object::FileMetaTable;
use indexmap::IndexMap;
use nu_protocol::{Span, Value};

use crate::convert::trim_string;

pub fn make_row_from_dicom_metadata(
    span: &Span,
    index_map: &mut IndexMap<String, Value>,
    meta: &FileMetaTable,
) {
    // TODO add more metadata

    index_map.insert("TransferSyntax".to_string(), Value::string(trim_string(&meta.transfer_syntax).to_owned(), *span));

    index_map.insert("MediaStorageSOPClassUID".to_string(), Value::string(trim_string(&meta.media_storage_sop_class_uid).to_owned(), *span));

    index_map.insert("MediaStorageSOPInstanceUID".to_string(), Value::string(trim_string(&meta.media_storage_sop_instance_uid).to_owned(), *span));
}
