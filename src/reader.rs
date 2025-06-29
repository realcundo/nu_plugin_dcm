use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::Path,
};

use dicom::object::{self as dicom_object, DefaultDicomObject};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not read Dicom object: {}", source))]
    Io { source: std::io::Error },

    #[snafu(display("Could not parse Dicom object: {}", source))]
    Dcm { source: dicom_object::ReadError },
}

pub fn read_dcm_file<P: AsRef<Path>>(path: P) -> Result<DefaultDicomObject, Error> {
    let path = path.as_ref();
    let input = BufReader::new(File::open(path).context(IoSnafu)?);
    read_dcm_stream(input)
}

pub fn read_dcm_stream<F: Read>(mut input: F) -> Result<DefaultDicomObject, Error> {
    // Read the first 132 bytes into a temporary buffer to check for the preamble.
    let mut buf = Vec::with_capacity(132);
    input
        .by_ref()
        .take(132)
        .read_to_end(&mut buf)
        .context(IoSnafu)?;

    if buf.len() == 132 && &buf[128..132] == b"DICM" {
        // "DICM" marker found. The data to parse starts with these 4 bytes.
        // We create a new reader by chaining the "DICM" marker from our buffer
        // with the rest of the original input stream.
        let reader = Cursor::new(&buf[128..]).chain(input);

        // Use the default OpenFileOptions to parse the File Meta Information.
        dicom_object::OpenFileOptions::new()
            .read_until(dicom::dictionary_std::tags::PIXEL_DATA)
            .read_preamble(dicom_object::file::ReadPreamble::Never)
            .from_reader(reader)
            .context(DcmSnafu)
    } else {
        // No "DICM" marker. The entire buffer is part of the dataset.
        // Create a reader from the buffer and chain it with the rest of the stream.
        let reader = Cursor::new(buf).chain(input);

        // Attempt to parse as a dataset without a preamble.
        dicom_object::OpenFileOptions::new()
            .read_until(dicom::dictionary_std::tags::PIXEL_DATA)
            .read_preamble(dicom_object::file::ReadPreamble::Never)
            .from_reader(reader)
            .context(DcmSnafu)
    }
}
