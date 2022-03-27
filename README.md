# nu_plugin_dcm

*Note that this plugin works with nu>=0.60. If you want to use nu<=0.44, use version 0.1.3 of this plugin.*

A [nushell](https://www.nushell.sh/) plugin to parse [Dicom](https://en.wikipedia.org/wiki/DICOM) objects.

This plugin is in the early stage of the development. It is usable but it might not be able to cope
with all Dicom objects. One notable limitation is that all Dicom objects are expected to have a preamble.

I'm still trying to figure out what is the most useful way of using this plugin. Please feel free to try it out, send feedback in [Discussions](https://github.com/realcundo/nu_plugin_dcm/discussions) or report problems in [Issues](https://github.com/realcundo/nu_plugin_dcm/issues).

## Usage
`dcm` plugin reads its input from single values or from specific columns:
- `dcm`: expects a string/filename or binary Dicom data
- `dcm $column_name`: reads a string/filename or binary Dicom data from `$column`. This is
  equivalent to `get $column | dcm`.

## Error handling

`dcm` plugin works in two modes:
- default, when errors are not skipped: each input is processed and errors are reported back to
  `nu` and they are not included in the output. This makes output potentially shorter than the
  input.
- skip errors using `-s`/`--silent-errors` flag: errors are output as empty values. This means that
  the output has exactly the same number of rows as the input. This mode is suitable for
  merging tables (e.g. table of files and table of parsed dicom objects).

## Known Limitations

- Dicom objects without a preamble and DCIM header will fail to load.
- PixelData is always skipped. For now I'm considering this to be a feature that speeds up Dicom parsing.


## Examples

### Output Dicom file as a table
```sh
echo file.dcm | dcm                # uses filename/string to specify which file to open
open file.dcm | dcm                # pass binary data to `dcm`
ls file.dcm | dcm name             # use `name` column as the filename
echo file.dcm | wrap foo | dcm foo # use `foo` column as the filename
open file.dcm | wrap foo | dcm foo # use `foo` column as binary data
```

### Dump Dicom file as a JSON/YAML document
```sh
open file.dcm | dcm | to json --indent 2
open file.dcm | dcm | to yaml
```

### Dump all Dicom files in the current directory to a JSON/YAML document
```sh
ls *.dcm | dcm name | to json --indent 2
ls *.dcm | dcm name | to yaml
```

### For each file in the current directory, show the filename, file size, SOP Instance UID, Modality and Pixel Spacing and sort by SOP Instance UID
PixelSpacing is an array with 2 values.

To flatten the array use `.0` and `.1` indices. `dcm` is
run using `--silent-errors`/`-s` to make sure that both `$files` and `dcm` have the same number of
rows. Without the flag the output of `dcm` could be shorted if Dicom object couldn't be parsed
resulting in incorrect merge.

```sh
let files = (ls | where type == file)

echo $files | select name size | merge { echo $files | dcm -s name | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size
```

`dcm name` is a shortcut for `get name | dcm`. The following commands are equivalent:
```sh
echo $files | select name size | merge { echo $files | dcm -s name | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size

echo $files | select name size | merge { echo $files.name | dcm -s | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size

echo $files | select name size | merge { echo $files | get name | dcm -s | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size
```

### Find all files in the current directory and subdirectories, parse them and group by Modality

Note that not all Dicom files have `(0008,0060)` Modality tag available. This will default missing
Modality tags to `???`.
```sh
ls **/* | where type == file | dcm name | default Modality '???' | group-by Modality  
```


## Installation

Build and install using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):
```sh
cargo install nu_plugin_dcm
```
and then register in nu via
```sh
register --encoding=json <PATH-TO-nu_plugin_dcm>/nu_plugin_dcm
```
Note that you **must** use `json` encoding. `capnp` is not supported yet.