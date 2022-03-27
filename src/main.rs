use nu_plugin::JsonSerializer;

mod convert;
mod dcm;
mod meta;
mod plugin;
mod reader;

fn main() {
    let mut plugin = plugin::DcmPlugin::default();

    // echo $files | merge { echo $files.name | dcm | get data | select Modality PixelSpacing.0 PixelSpacing.1 } | sort-by Modality name

    // use JsonSerializer because Cap'nProto doesn't support CellPath yet (https://github.com/nushell/nushell/issues/5023 and https://github.com/nushell/nushell/pull/4920)
    nu_plugin::serve_plugin(&mut plugin, JsonSerializer {});
}
