use std::{
    fs::File,
    io::{BufReader, ErrorKind, Read, Seek, SeekFrom},
    path::Path,
};

use dicom::object::{self as dicom_object, DefaultDicomObject};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Could not read Dicom object: {}", source))]
    Io { source: std::io::Error },

    #[snafu(display("Could not read Dicom preamble"))]
    Preamble,

    #[snafu(display("Could not parse Dicom object: {}", source))]
    Dcm { source: dicom_object::Error },

    #[snafu(display("Could not parse Dicom object (no preamble?): {}", source))]
    DcmNoPreamble { source: dicom_object::Error },
}

pub fn read_dcm_file<P: AsRef<Path>>(path: P) -> Result<DefaultDicomObject, Error> {
    let path = path.as_ref();
    let input = BufReader::new(File::open(path).with_context(|_| IoSnafu)?);
    read_dcm_stream(input)
}

pub fn read_dcm_stream<F: Seek + Read>(mut input: F) -> Result<DefaultDicomObject, Error> {
    // TODO use lower level Dicom functions to avoid seeking back and forth and double-wrapping BufReaders

    // read the first 128 + 4 bytes and check if DICM
    let mut buf = [0u8; 128 + 4];

    match input.read_exact(&mut buf) {
        Ok(_) => {
            // check if DICM
            if buf[128..132] == [b'D', b'I', b'C', b'M'] {
                // need to rewind back 4 to get to the beginning of DCIM again
                input
                    .seek(SeekFrom::Current(-4))
                    .with_context(|_| IoSnafu)?;

                return read_dcm_stream_without_pixel_data(input).with_context(|_| DcmSnafu);
            }
        }
        Err(e) => {
            // if seek error, fall through and try to read without preamble, otherwise fail now
            if e.kind() != ErrorKind::UnexpectedEof {
                return Err(Error::Io { source: e });
            }
        }
    }

    // Rewind to the start and try to read without the preamble
    input.seek(SeekFrom::Start(0)).with_context(|_| IoSnafu)?;

    // TODO this will always fail -- dicom.rs needs DCIM magic to read meta
    read_dcm_stream_without_pixel_data(input).with_context(|_| DcmNoPreambleSnafu)
}

fn read_dcm_stream_without_pixel_data<F: Read>(
    input: F,
) -> Result<DefaultDicomObject, dicom_object::Error> {
    dicom_object::OpenFileOptions::new()
        .read_until(dicom::dictionary_std::tags::PIXEL_DATA)
        .read_preamble(dicom_object::file::ReadPreamble::Never)
        .from_reader(input)
}
