#!/usr/bin/env python3

from pydicom.dataset import FileDataset, FileMetaDataset
import pydicom
import os

ASSETS_DIR = os.path.dirname(__file__)


class ExplicitVRLittleEndian:
    uid = pydicom.uid.ExplicitVRLittleEndian
    little_endian = True
    implicit_vr = False


class ImplicitVRLittleEndian:
    uid = pydicom.uid.ImplicitVRLittleEndian
    little_endian = True
    implicit_vr = True


class ExplicitVRBigEndian:
    uid = pydicom.uid.ExplicitVRBigEndian
    little_endian = False
    implicit_vr = False


class DeflatedExplicitVRLittleEndian:
    uid = pydicom.uid.DeflatedExplicitVRLittleEndian
    little_endian = True
    implicit_vr = False


for transfer_syntax in [
    ExplicitVRLittleEndian,
    ImplicitVRLittleEndian,
    ExplicitVRBigEndian,
    # DeflatedExplicitVRLittleEndian, # TODO
]:
    for preamble in [False, True]:
        suffix = "Preamble" if preamble else "NoPreamble"
        name = f"{transfer_syntax.__name__}-{suffix}"
        filename = f"{name}.dcm"

        file_meta = FileMetaDataset()
        file_meta.MediaStorageSOPClassUID = "1.2.840.10008.5.1.4.1.1.2"
        file_meta.MediaStorageSOPInstanceUID = "1.2.3"
        file_meta.ImplementationClassUID = "1.2.3.4"
        file_meta.FileMetaInformationGroupLength = 0  # will be updated

        preamble_data = b"\0" * (128 if preamble else 0)

        ds = FileDataset(filename, {}, file_meta=file_meta, preamble=preamble_data)
        ds.file_meta.TransferSyntaxUID = transfer_syntax.uid
        ds.is_little_endian = transfer_syntax.little_endian
        ds.is_implicit_VR = transfer_syntax.implicit_vr

        ds.PatientName = name

        ds.save_as(os.path.join(ASSETS_DIR, filename))
