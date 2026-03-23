# AGENTS.md — seq-rs Codebase Guide

This document is a reference for AI coding agents and contributors working in this repository. It describes the project purpose, architecture, module structure, data flow, testing approach, and development conventions.

---

## Project Overview

**seq-rs** (package name: `sequencer`) is a command-line tool written in Rust that converts a simple plain-text sequence diagram DSL into a rendered PNG image.

The typical workflow is:

```
text input (file / stdin / built-in example)
  → DocumentParser   (parse lines into typed LineContents)
  → ParticipantParser (extract participants + compute geometry)
  → InteractionParser (extract arrows / interactions)
  → Diagram           (assembled data model)
  → Render            (raqote 2D drawing → PNG file)
```

---

## Technology Stack

| Concern | Crate / Tool |
|---|---|
| Language | Rust (edition 2018) |
| Build & package manager | Cargo |
| CLI argument parsing | `clap` 3.0.0-beta.2 |
| 2D rendering | `raqote` 0.8 (with `pathfinder_geometry`) |
| Font rasterisation & layout | `fontdue` 0.5 |
| Geometry primitives | `euclid`, `pathfinder_geometry` |
| Regex parsing | `regex` 1.5 |
| Iterator utilities | `itertools` 0.10 |
| Lazy statics | `lazy_static` 1.4 |
| Logging | `log` + `pretty_env_logger` |
| Benchmarking | `criterion` 0.3 |
| CI | GitHub Actions |
| Code coverage | `cargo-tarpaulin` + Codecov |

---

## Directory Structure

```
seq-rs/
├── Cargo.toml                   # Package metadata and dependencies
├── README.md
├── AGENTS.md                    # This file
├── assets/
│   ├── OpenSans-Regular.ttf     # Bundled fonts (included at compile time)
│   ├── Roboto-Black.ttf
│   └── Roboto-Thin.ttf          # Currently used by Theme::default()
├── benches/
│   ├── benchmark_diagram_parsing.rs   # Criterion benchmarks: participant + interaction parsing
│   ├── benchmark_document_parsing.rs  # Criterion benchmarks: document parsing
│   ├── benchmark_rendering.rs         # (disabled in Cargo.toml)
│   └── text_benchmarks.rs             # (disabled in Cargo.toml)
├── src/
│   ├── main.rs       # Binary entry point
│   ├── cli.rs        # CLI argument definitions (clap)
│   ├── lib.rs        # Library root; module exports; type aliases
│   ├── mod.rs        # (empty / legacy, all content commented out)
│   ├── model.rs      # Core domain types
│   ├── diagram.rs    # Diagram struct + parse() orchestration
│   ├── theme.rs      # Theme (fonts, sizes, spacing)
│   ├── parsing/
│   │   ├── mod.rs         # Re-exports document, interaction, participant
│   │   ├── document.rs    # DocumentParser: text → Vec<Line>
│   │   ├── interaction.rs # InteractionParser: Vec<Line> → InteractionSet
│   │   └── participant.rs # ParticipantParser: Vec<Line> → ParticipantSet (with geometry)
│   └── rendering/
│       ├── mod.rs    # Render traits, RenderContext, Diagram::render(), Participant::render()
│       └── text.rs   # measure_string() and draw_text() via fontdue
└── .github/
    └── workflows/
        ├── build_and_test.yml  # cargo build + cargo test on push/PR to main
        └── codecov.yml         # cargo-tarpaulin coverage upload to Codecov
```

---

## Source Module Details

### `src/main.rs` — Binary entry point

- Initialises `pretty_env_logger` for log output controlled by `RUST_LOG`.
- Calls `parse_cli_args()` to read CLI flags via `cli.rs`.
- Calls `load_data()` to read lines from a file path, stdin, or the built-in example string.
- Runs the full parsing + rendering pipeline.
- Warns about any lines classified as `LineContents::Invalid`.

### `src/cli.rs` — CLI argument parsing

Three arguments are defined with `clap`:

| Argument | Flag | Notes |
|---|---|---|
| `input` | `-f` / `--file` | Path to a `.seq` input file |
| `example` | `-e` | Use the built-in example diagram (conflicts with `--file`) |
| `output` | (positional) | Required output PNG file path |

### `src/lib.rs` — Library root

- Declares public modules: `diagram`, `model`, `parsing`, `rendering`, `theme`.
- Defines two type aliases used throughout the codebase:
  - `InteractionSet = Vec<Interaction>`
  - `ParticipantSet = HashSet<Participant>`

### `src/model.rs` — Domain types

Key types:

| Type | Purpose |
|---|---|
| `Line` | A parsed line: `line_number`, `line_data` (raw string), `line_contents` |
| `LineContents` | Enum: `Empty`, `Comment`, `MetaData(MetaDataType)`, `Interaction(From, To)`, `InteractionWithMessage(From, To, Msg)`, `Invalid` |
| `MetaDataType` | Enum: `Style`, `FontSize`, `Title`, `Author`, `Date`, `Invalid` |
| `Participant` | Named participant with `index`, `active_from`, `active_to`, `rect: Rect` |
| `Interaction` | Directional arrow: `index`, `from_participant`, `to_participant`, `interaction_type`, `message` |
| `InteractionType` | Enum: `L2R`, `R2L`, `SelfRef` |
| `Message` | Newtype wrapper around `String` |
| `Config` | `input_source: Source`, `output_path: String` |
| `Source` | Enum: `StdIn`, `File(String)`, `Example` |

### `src/diagram.rs` — Diagram assembly

`Diagram::parse(document: Document, theme: Theme) -> Diagram` orchestrates the second and third parsing stages:

1. Calls `ParticipantParser::parse(&document.lines, &theme)` → `ParticipantSet`
2. Calls `InteractionParser::parse(&document.lines, &participants)` → `InteractionSet`
3. Returns `Diagram { theme, header, interactions, participants, config }`.

### `src/theme.rs` — Visual theme

`Theme` holds all visual parameters:

- `title_font`, `body_font` — `fontdue::Font` instances loaded from embedded TTF bytes (`include_bytes!`).
- `title_font_px`, `partic_font_px`, `message_font_px` — font sizes in pixels.
- `document_border_width`, `partic_padding`, `partic_h_gap` — spacing values.

`Theme::default()` uses `Roboto-Thin.ttf` for both title and body fonts.

### `src/parsing/document.rs` — Document parsing

`DocumentParser::parse(lines: &[String], config: Config) -> Document`

Classifies each input line by inspecting its content:

- Empty → `LineContents::Empty`
- Starts with `#` → `LineContents::Comment`
- Starts with `:` → metadata via `parse_metadata()`
- Contains `->` → interaction via `parse_interaction()` using the regex `^(.+)\s+-+>+\s+([^:]+):?(.*)$`
- Otherwise → `LineContents::Invalid`

`parse_metadata` recognises `:theme`, `:title`, `:author`, `:date` keywords.

### `src/parsing/participant.rs` — Participant parsing

`ParticipantParser::parse(document: &[Line], theme: &Theme) -> ParticipantSet`

Single-pass over interaction lines (ignoring all other line types):

1. Discovers participants in order of first appearance, assigning sequential indices.
2. Tracks `active_from` (first interaction index) and `active_to` (last interaction index) for each participant.
3. Measures each participant's label width using `rendering::text::measure_string()` to compute `Rect` geometry.
4. Returns a `HashSet<Participant>`.

### `src/parsing/interaction.rs` — Interaction parsing

`InteractionParser::parse(document: &[Line], participants: &HashSet<Participant>) -> InteractionSet`

Filters to only interaction lines, looks up the `Participant` objects by name, determines `InteractionType` by comparing participant indices (`L2R`, `R2L`, `SelfRef`), and returns a `Vec<Interaction>`.

### `src/rendering/mod.rs` — Rendering

- `Diagram::render()` — computes overall image size, creates a `RenderContext` (white background `DrawTarget`), renders all participants, and writes a PNG using `DrawTarget::write_png()`.
- `Participant::render()` — draws a stroked rectangle and the participant label via `draw_text()`.
- `Diagram::size()` — derives `width` and `height` from participant positions and interaction count.
- `RenderContext` — wraps a `DrawTarget` and the active `Theme`.

### `src/rendering/text.rs` — Text utilities

- `measure_string(theme, content, px) -> Rect` — uses `fontdue::layout::Layout` to compute the bounding box of a string without drawing it.
- `draw_text(rc, content, x, y, px)` — lays out and rasterises each glyph, composites it onto the `DrawTarget` via `draw_image_at`.
- `rgb_to_u32(r, g, b, a) -> u32` — packs colour channels into a `u32` pixel value.

---

## Input DSL Syntax

```
# Lines starting with '#' are comments and are ignored.

# Metadata directives (colon-prefixed):
:theme Default
:title My Diagram Title
:author Author Name
:date 2024-01-01
# Note: ':date' requires at least one trailing character after the keyword
# because the parser uses split_once(whitespace). A bare ':date' line with
# no trailing content will be classified as Invalid by the current implementation.

# Interaction lines:
Client -> Server: Request message
Server -> Server: Self-reference
Server --> Client: Reply (dashed)
Server ->> Service: Async arrow
Service -->> Server: Async reply
```

The interaction regex `^(.+)\s+-+>+\s+([^:]+):?(.*)$` captures:
- Group 1: from-participant name
- Group 2: to-participant name
- Group 3: optional message (after `:`)

Arrow variants (`->`, `-->`, `->>`, `-->>`) are all matched by the same regex pattern (they are not yet differentiated in the model).

---

## Building and Running

```bash
# Build
cargo build

# Build (release)
cargo build --release

# Run with a file
cargo run -- --file path/to/diagram.seq output.png

# Run with the built-in example
cargo run -- -e output.png

# Run with stdin
echo "Client -> Server: Hello" | cargo run -- output.png

# Enable logging
RUST_LOG=info cargo run -- -e output.png
RUST_LOG=debug cargo run -- -e output.png
```

---

## Testing

Tests are written as inline `#[test]` functions within source files. Run them with:

```bash
cargo test
cargo test --verbose
```

### Active tests

| File | Test name | What it covers |
|---|---|---|
| `src/parsing/document.rs` | `test_parse_metadata` | `:title` metadata parsing with and without whitespace |
| `src/parsing/document.rs` | `test_document_parser_with_invalid` | Mixed valid/invalid lines produce correct `LineContents` |
| `src/parsing/document.rs` | `test_document_parser` | Full document with metadata, interactions, empty lines |
| `src/rendering/text.rs` | `test_measure_text` | `measure_string` returns correct `Rect` for single and double character strings |

### Commented-out tests

Several tests in `src/parsing/interaction.rs` and `src/parsing/participant.rs` are commented out because `Participant` now carries a `rect: Rect` field that requires `Theme` to compute. These tests can be re-enabled by constructing a `Theme::default()` and passing it to `ParticipantParser::parse`.

---

## Benchmarks

Benchmarks use [Criterion](https://github.com/bheisler/criterion.rs). Two benchmark suites are active:

```bash
cargo bench
```

| File | Benchmark name | What it measures |
|---|---|---|
| `benches/benchmark_document_parsing.rs` | `parsing document` | `DocumentParser::parse` throughput |
| `benches/benchmark_diagram_parsing.rs` | `parsing participants` | `ParticipantParser::parse` throughput |
| `benches/benchmark_diagram_parsing.rs` | `parsing interactions` | `InteractionParser::parse` throughput |

Two benchmark files (`benchmark_rendering.rs`, `text_benchmarks.rs`) exist but are commented out in `Cargo.toml`.

---

## CI / CD

### `.github/workflows/build_and_test.yml`

Runs on every push and pull request to `main`:
1. `cargo build --verbose`
2. `cargo test --verbose`

### `.github/workflows/codecov.yml`

Runs on every push:
1. `cargo-tarpaulin` collects line coverage.
2. Results are uploaded to [Codecov](https://codecov.io/gh/rsouth/seq-rs).

---

## Key Design Decisions and Notes

- **Library + binary split**: The crate is structured as a library (`lib.rs`) consumed by the binary (`main.rs`). This enables benchmarks and future integration tests to import public types and parsers directly.
- **Fonts are embedded at compile time**: `include_bytes!` bakes the TTF files into the binary so no runtime font discovery is needed.
- **`HashSet<Participant>` for participants**: Participants are stored in a `HashSet` and sorted by `index` at render time. This avoids duplicates when a participant appears in multiple interactions.
- **Single-pass participant discovery**: `ParticipantParser` assigns indices in order of first appearance in the document, which determines left-to-right visual ordering.
- **`raqote` + `fontdue` for rendering**: `raqote` provides a software 2D canvas (no GPU required); `fontdue` handles font loading, layout, and glyph rasterisation.
- **Interactions not yet fully differentiated**: Arrow variants (`->`, `-->`, `->>`, `-->>`) are all parsed the same way by the current regex; the dashed/async distinction is not yet surfaced in the model.
- **`LineContents::Invalid` is non-fatal**: Invalid lines produce a `warn!` log but do not stop diagram generation.
- **`Cargo.lock` is gitignored**: The project is a binary crate. Per Rust best practices, binary projects should commit `Cargo.lock` to guarantee reproducible builds. The current `.gitignore` excludes it; consider removing that exclusion.

---

## Extending the Codebase

| Task | Where to look |
|---|---|
| Add a new metadata directive | `parsing/document.rs` → `parse_metadata()` and `model.rs` → `MetaDataType` |
| Differentiate arrow styles | `model.rs` → new field on `Interaction` or extend `InteractionType`; update `parsing/document.rs` regex/parsing |
| Render interaction arrows | `rendering/mod.rs` → implement `Render for Interaction` |
| Add a new theme | `theme.rs` → new `Theme` variant or named constructor |
| Add a new CLI flag | `cli.rs` → add `Arg`; `main.rs` → read and propagate to `Config` |
| Add output formats | `rendering/mod.rs` → `Diagram::render()` — swap `write_png` for another encoder |
