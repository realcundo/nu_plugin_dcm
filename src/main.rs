use nu_plugin::MsgPackSerializer;

mod convert;
mod dcm;
mod dicomweb;
mod meta;
mod plugin;
mod reader;

fn main() {
    let plugin = plugin::DcmPlugin::default();

    // echo $files | merge { echo $files.name | dcm | get data | select Modality PixelSpacing.0 PixelSpacing.1 } | sort-by Modality name

    nu_plugin::serve_plugin(&plugin, MsgPackSerializer);
}
