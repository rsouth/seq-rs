use std::io;
use std::io::Read;
use std::time::Instant;

use clap::ArgMatches;
use itertools::Itertools;
use log::{info, warn};

use sequencer::diagram::Diagram;
use sequencer::model::{Config, LineContents, Source};
use sequencer::parsing::document::DocumentParser;
use sequencer::theme::Theme;

mod cli;

fn read_from_stdin() -> Vec<String> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut lines = Vec::new();

    let mut line = String::new();
    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 {
            break;
        }
        lines.push(line.clone());
        line.clear();
    }

    lines
}

fn main() {
    pretty_env_logger::init();
    let instant = Instant::now();

    let config = parse_cli_args();
    println!("Config: {:?}", config);

    // load in data from file/stdin/etc
    let data = load_data(&config.input_source);
    println!("{:?}", data);

    let document = DocumentParser::parse(&data, config);
    info!("Document: {:#?}", document);

    document
        .lines
        .iter()
        .filter(|p| p.line_contents == LineContents::Invalid)
        .for_each(|p| {
            warn!("Warning: Line {} is bad: {}", p.line_number, p.line_data);
        });

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

fn parse_cli_args() -> Config {
    let cli_options = cli::parse_args();
    let input_source = resolve_input_source(&cli_options);

    let output_path = cli_options
        .get_one::<String>(cli::OUTPUT_FILE)
        .cloned()
        .unwrap_or_default();
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
    if options.contains_id(cli::EXAMPLE) && *options.get_one::<bool>(cli::EXAMPLE).unwrap_or(&false) {
        Source::Example
    } else if let Some(file) = options.get_one::<String>(cli::INPUT_FILE) {
        Source::File(file.clone())
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
        .map(|p| p.to_string())
        .collect_vec()
}
