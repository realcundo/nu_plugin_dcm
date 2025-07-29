# nu_plugin_dcm

*Compatibility Matrix*
| nu version | nu_plugin_dcm version |
|------------|-----------------------|
| 0.106.x    | 0.4.0                 |
| 0.105.x    | 0.3.2                 |
| 0.60.x     | 0.1.8                 |

*Also note that this version is not fully compatible with 0.2.x and earlier versions of this plugin. The main difference is that `ls *.dcm | dcm name` doesn't work anymore.
Use `ls *.dcm | dcm`, `ls *.dcm | get name` or `ls *.dcm | select name type | dcm` instead. `dcm name` is now reserved for the future to select dicom tags.*

A [nushell](https://www.nushell.sh/) plugin to parse [DICOM](https://en.wikipedia.org/wiki/DICOM) objects.

This plugin is in the early stage of the development. It is usable but it might not be able to cope
with all DICOM objects or DICOMweb records. See [Known limitations for details](#known-limitations).

I'm still trying to figure out what is the most useful way of using this plugin. Please feel free to try it out,
send feedback in [Discussions](https://github.com/realcundo/nu_plugin_dcm/discussions) or report problems in [Issues](https://github.com/realcundo/nu_plugin_dcm/issues).

## Usage
`dcm` plugin reads its input from single values, or from list of values:
- `dcm`: expects a string/filename, file record (must contain `name` and `type`), DICOMweb record, or binary DICOM data
  - `ls *.dcm | dcm`: process a list of files, resulting in a list of dicom records
  - `ls *.dcm | select name type | dcm`: process a list of files specified by their filename, resulting in a list of dicom records
  - `open --raw file.dcm | into binary | dcm`: process a binary stream, resulting in a dicom record
  - `open dicomweb.json | dcm`: process a dicomweb record, resulting in a dicom record

See Examples for more details.

## Error handling

`dcm` plugin works in two modes:
- default, when errors are reported as error rows, reported by nu,
- when `--error` option is used, errors are reported in provided column. If there were no errors, the column value is empty.

## Known Limitations

- DICOM objects without a preamble and DCIM header will fail to load.
- PixelData is always skipped. For now I'm considering this to be a feature that speeds up DICOM parsing.
- `dcm` can process binary data. You can pass it directly to `dcm` as `open --raw file.dcm | dcm`. However, when passing a list of binary streams,
  `nushell` will try to convert it to a list of strings. To work around this, use `into binary`, e.g.:
  ```sh
  [(open --raw file1.dcm | into binary), (open --raw file2.dcm | into binary)] | dcm
  ```

  Without `into binary`, `dcm` would see a list of strings, assuming it's a list of filenames.
- For DICOMweb inputs, only the first of "Alphabetic", "Ideographic", "Phonetic" Patient Names is extracted.
- For DICOMweb inputs, `BulkDataURI` and `InlineBinary` are not extracted and `nothing` is returned as their values.


## Examples

### Output DICOM file as a record/table (list of records)
```sh
echo file.dcm | dcm                      # uses filename/string to specify which file to open
"file.dcm" | dcm                         # same asa above, uses filename/string to specify which file to open
open --raw file.dcm | dcm                # pass (hopefully) binary data to `dcm`
open --raw file.dcm | into binary | dcm  # pass binary data to `dcm`
ls file.dcm | dcm                        # use file records as the filename
ls file.dcm | select name type | dcm     # use file record-like records as the filename
# ls file.dcm | select name | dcm        # fails because the record only contains `name` field.
ls file.dcm | get name | dcm             # use a list of filenames (list of strings, rather than a list of records)
```

### Dump DICOM file as a JSON/YAML document
```sh
open -r file.dcm | dcm | to json --indent 2
open -r file.dcm | dcm | to yaml
```

### Dump all DICOM files in the current directory to a JSON/YAML document
```sh
ls *.dcm | dcm | to json --indent 2
ls *.dcm | dcm | to yaml
```

### Find all files in the current directory and subdirectories, parse them and group by Modality

```sh
ls **/* |
  where type == file |
  dcm -e error |
  where error == "" |
  select --ignore-errors SOPInstanceUID Modality |
  group-by Modality
```

### For each file in the current directory, show the filename, file size, SOP Instance UID and Modality, and sort by SOP Instance UID
```sh
let files = (ls | where type == file)

$files |
select name size |
merge ($files |
  dcm -e error |
  select --ignore-errors SOPInstanceUID Modality error
) |
sort-by size
```
Note that when a file cannot be parsed, it won't have `SOPInstanceUID`, etc. columns. Without `--ignore-errors` `select`
would fail since selected columns are missing. Another option would be using `default "" SOPInstanceUID` to add values
for missing columns.)


### For each file in all subdirectories, show filename, file size, SHA256 hash of the file, SOP Instance UID and a DICOM parsing error, if any
Use `par-each` to process files in parallel:
```sh
ls **/* | where type == file |
  par-each { |it| {
    name: $it.name,
    size: $it.size,
    sha256: (open --raw $it.name | hash sha256),
    dcm: ($it.name | dcm -e error)
   } } |
   select --ignore-errors name size sha256 dcm.Modality dcm.SOPInstanceUID dcm.error |
   sort-by name
```


## Installation

Build and install using [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html):

```sh
cargo install nu_plugin_dcm
```

and then register in nu, e.g.

```nu
plugin add ~/.cargo/bin/nu_plugin_dcm
```

To start using it without restarting nu, you can [import it](https://www.nushell.sh/book/plugins.html#importing-plugins):

```nu
plugin use ~/.cargo/bin/nu_plugin_dcm
```
