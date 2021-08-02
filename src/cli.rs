use clap::{crate_version, App, AppSettings, Arg, ArgMatches};

pub const INPUT_FILE: &str = "input";
pub const EXAMPLE: &str = "example";
pub const OUTPUT_FILE: &str = "output";

pub(crate) fn cli() -> ArgMatches {
    App::new("MyApp")
        .setting(AppSettings::ColoredHelp)
        .about("does awesome things")
        .version(crate_version!())
        .after_help("Check github.com/rsouth/seq-rs for the latest release")
        .arg(
            Arg::new(INPUT_FILE)
                .short('f')
                .long("file")
                .takes_value(true),
        )
        .arg(
            Arg::new(EXAMPLE)
                .short('e')
                .about("use example file")
                .conflicts_with(INPUT_FILE),
        )
        // .arg("<output> 'sets an output file' 'test'")
        .arg(
            Arg::new(OUTPUT_FILE)
                .last(true)
                .about("sets an output file")
                // .long_about("can be either with or without .png extension")
                .required(true), // .required_unless_present() // todo when we add 'validate' mode
        )
        // ideas: override theme, select output file type (png vs jpg, does anyone care?)
        .get_matches()
}
