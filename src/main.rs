mod cli;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use sequencer::diagram::Diagram;
use sequencer::parsing::document::DocumentParser;
use sequencer::theme::Theme;
use std::time::Instant;

use clap::ArgMatches;
use itertools::Itertools;
use std::io;
use std::io::Read;

fn read_from_stdin() -> Vec<String> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock(); // locking is optional

    let mut line = String::new();
    let mut lines = Vec::new();

    // Could also `match` on the `Result` if you wanted to handle `Err`
    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 {
            break;
        }
        // println!("{}", line);
        lines.push(line.clone());
        line.clear();
    }

    lines
}

#[derive(Debug)]
enum Source {
    StdIn,
    File(String),
    Example,
}

#[derive(Debug)]
struct Config {
    pub input_source: Source,
    pub output_path: String,
}

fn main() {
    pretty_env_logger::init();
    let instant = Instant::now();

    let config = ppaarrssee_aarrggss();
    println!("Config: {:?}", config);

    // load in data from file/stdin/etc
    let data = load_data(&config.input_source);
    println!("{:?}", data);

    let document = DocumentParser::parse(data);
    info!("Document: {:#?}", document);

    let theme = Theme::default();
    let diagram = Diagram::parse(document, theme);
    info!("Diagram: {:#?}", diagram);

    diagram.render();

    info!(
        "Finished in {} micros ({}ms)",
        instant.elapsed().as_micros(),
        instant.elapsed().as_millis()
    );
}

fn ppaarrssee_aarrggss() -> Config {
    // parse CLI args
    let cli_options = cli::parse_args();
    let input_source = resolve_input_source(&cli_options);

    // todo 'document config' here - can override things like the :theme (from args) etc.
    let output_path = cli_options.value_of(cli::OUTPUT_FILE).unwrap().to_string();
    Config {
        input_source,
        output_path,
    }
}

fn load_data(src: &Source) -> Vec<String> {
    match src {
        Source::StdIn => {
            println!("Reading from stdin");
            read_from_stdin()
        }
        Source::File(file_name) => {
            println!("Reading from file {}", file_name);
            std::fs::read_to_string(file_name)
                .unwrap()
                .lines()
                .into_iter()
                .map(|p| p.to_string())
                .collect_vec()
        }
        Source::Example => {
            println!("Using example file");
            get_text()
        }
    }
}

fn resolve_input_source(options: &ArgMatches) -> Source {
    if options.is_present(cli::EXAMPLE) {
        Source::Example
    } else if options.is_present(cli::INPUT_FILE) {
        Source::File(options.value_of(cli::INPUT_FILE).unwrap().to_string())
    } else {
        Source::StdIn
    }
}

pub fn get_text() -> Vec<String> {
    ":theme Default
     :title Example Sequence Diagram
     :author Mr. Sequence Diagram
     :date
    
     # diagram
     Client -> Server: Request
     Server -> Server: Parses request
     Server ->> Service: Query
     Service -->> Server: Data
     Server --> Client: Response
     Left -> Right
     # {AMPS} -> Client: "
        .lines()
        .into_iter()
        .map(|p| p.to_string())
        .collect_vec()
}
