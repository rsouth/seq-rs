use clap::{crate_version, Arg, ArgMatches, Command};

pub const INPUT_FILE: &str = "input";
pub const EXAMPLE: &str = "example";
pub const OUTPUT_FILE: &str = "output";

pub(crate) fn parse_args() -> ArgMatches {
    Command::new("Sequencer")
        .about("does awesome things")
        .version(crate_version!())
        .after_help("Check github.com/rsouth/seq-rs for the latest release")
        .arg(
            Arg::new(INPUT_FILE)
                .short('f')
                .long("file")
                .num_args(1)
                .required(false),
        )
        .arg(
            Arg::new(EXAMPLE)
                .short('e')
                .help("use example file")
                .num_args(0)
                .conflicts_with(INPUT_FILE),
        )
        .arg(
            Arg::new(OUTPUT_FILE)
                .help("sets an output file")
                .required(true),
        )
        .get_matches()
}
