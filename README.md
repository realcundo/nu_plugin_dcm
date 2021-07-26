# nu_plugin_dcm

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
open file.dcm | dcm | to json --pretty 2
open file.dcm | dcm | to yaml
```

### Dump all Dicom files in the current directory to a JSON/YAML document
```sh
ls *.dcm | dcm name | to json --pretty 2
ls *.dcm | dcm name | to yaml
```

### For each file in the current directory, show the filename, file size, SOP Instance UID, Modality and Pixel Spacing and sort by SOP Instance UID
PixelSpacing is an array with 2 values.

To flatten the array use `.0` and `.1` indices. `dcm` is
run using `--silent-errors`/`-s` to make sure that both `$files` and `dcm` have the same number of
rows. Without the flag the output of `dcm` could be shorted if Dicom object couldn't be parsed
resulting in incorrect merge.

```sh
let files = (ls | where type == File)

echo $files | select name size | merge { echo $files | dcm -s name | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size
```

`dcm name` is a shortcut for `get name | dcm`. The following commands are equivalent:
```sh
echo $files | select name size | merge { echo $files | dcm -s name | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size

echo $files | select name size | merge { echo $files.name | dcm -s | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size

echo $files | select name size | merge { echo $files | get name | dcm -s | select SOPInstanceUID Modality PixelSpacing.0 PixelSpacing.1 } | sort-by size
```

### Find all files in the current directory and subdirectories, parse them and output a histogram of modalities

Note that not all Dicom files have `(0008,0060)` Modality tag available. This will default missing
Modality tags to `???`.
```sh
find . -type f | lines | dcm | default Modality '???' | histogram Modality
```


## Installation

Build and install in the same directory as `nu` command. Alternatively add the installation path to your
[`plugin_dirs` in your `config.toml`](https://www.nushell.sh/book/configuration.html#nushell-configuration-with-config-toml).
```sh
cargo install nu_plugin_dcm
```
Note that at the time of writing the Nu documentation is out of date and `PATH` env variable is NOT used to discover plugins.