# nu_plugin_dcm

*Note that this plugin works with nu>=0.60. If you want to use nu<=0.44, use version 0.1.3 of this plugin.*

A [nushell](https://www.nushell.sh/) plugin to parse [Dicom](https://en.wikipedia.org/wiki/DICOM) objects.

This plugin is in the early stage of the development. It is usable but it might not be able to cope
with all Dicom objects. One notable limitation is that all Dicom objects are expected to have a preamble.

I'm still trying to figure out what is the most useful way of using this plugin. Please feel free to try it out,
send feedback in [Discussions](https://github.com/realcundo/nu_plugin_dcm/discussions) or report problems in [Issues](https://github.com/realcundo/nu_plugin_dcm/issues).

## Usage
`dcm` plugin reads its input from single values or from specific columns:
- `dcm`: expects a string/filename or binary Dicom data
- `dcm $column_name`: reads a string/filename or binary Dicom data from `$column`. This is
  equivalent to `get $column | dcm`.

## Error handling

`dcm` plugin works in two modes:
- default, when errors are reported as error rows,
- in custom columns when `--error` option is used. This will report all errors in the specified column. Empty column value means no error.

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


### Find all files in the current directory and subdirectories, parse them and group by Modality

```sh
ls **/* |
  where type == file |
  dcm name -e error |
  where error == "" |
  group-by Modality
```

### For each file in the current directory, show the filename, file size, SOP Instance UID, Modality and Pixel Spacing and sort by SOP Instance UID
PixelSpacing is an array with 2 values.

To flatten the array use `.0` and `.1` indices.

```sh
let files = (ls | where type == file)

echo $files |
  select name size |
  merge {
    echo $files |
    dcm name -e error |
    default "" SOPInstanceUID |
    select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 error
  } |
  sort-by size
```
Note that when a file cannot be parsed, it won't have `SOPInstanceUID` column. The `default` commands makes sure that `select` can find the column.

You can also use `each` and `par-each` like in the following example.


### For each file in all subdirectories, show filename, file size, SHA256 hash of the file, SOP Instance UID and a Dicom parsing error, if any
Use `par-each` to process files in parallel:
```sh
ls **/* |
  par-each { |it| {
    name: $it.name,
    size: $it.size,
    sha256: (open $it.name | hash sha256),
    dcm: ($it.name | dcm -e error)
   } } |
   select name size sha256 dcm.Modality dcm.SOPInstanceUID dcm.error |
   sort-by name
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