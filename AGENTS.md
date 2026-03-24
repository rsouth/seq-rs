# AGENTS.md — seq-rs Codebase Guide

This document is a reference for AI coding agents and contributors working in this repository. It describes the project purpose, architecture, module structure, data flow, testing approach, and development conventions.

---

## Project Overview

**seq-rs** (package name: `sequencer`) is a command-line tool written in Rust that converts a simple plain-text sequence diagram DSL into a rendered PNG image.

The typical workflow is:

```
text input (file / stdin / built-in example)
  → DocumentParser    (parse lines into typed LineContents)
  → ParticipantParser (extract participants + compute geometry)
  → InteractionParser (extract arrows / interactions)
  → Diagram           (assembled data model)
  → Render            (raqote 2D drawing → PNG file)
```

---

## Technology Stack

| Concern | Crate / Tool |
|---|---|
| Language | Rust (edition 2021) |
| Build & package manager | Cargo |
| CLI argument parsing | `clap` 4 (with `derive` + `cargo` features) |
| 2D rendering | `raqote` 0.8 (with `pathfinder_geometry`) |
| Font rasterisation & layout | `fontdue` 0.9 |
| Geometry primitives | `euclid`, `pathfinder_geometry` |
| Regex parsing | `regex` 1 |
| Iterator utilities | `itertools` 0.14 |
| Logging | `log` 0.4 + `pretty_env_logger` 0.5 |
| Benchmarking | `criterion` 0.8 |
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
│   └── Roboto-Thin.ttf          # Used by Theme::default()
├── benches/
│   ├── benchmark_diagram_parsing.rs   # Criterion: participant + interaction parsing
│   ├── benchmark_document_parsing.rs  # Criterion: document parsing
│   ├── benchmark_rendering.rs         # Criterion: full diagram parse+render cycle
│   └── text_benchmarks.rs             # Criterion: measure_string, rgb_to_u32
├── docs/
│   └── example.png              # Example output image (referenced by README)
├── src/
│   ├── main.rs       # Binary entry point
│   ├── cli.rs        # CLI argument definitions (clap)
│   ├── lib.rs        # Library root; module exports; type aliases
│   ├── mod.rs        # Legacy file — all content commented out; not used
│   ├── model.rs      # Core domain types
│   ├── diagram.rs    # Diagram struct + parse() orchestration
│   ├── theme.rs      # Theme (fonts, sizes, spacing)
│   ├── parsing/
│   │   ├── mod.rs         # Re-exports document, interaction, participant
│   │   ├── document.rs    # DocumentParser: text → Vec<Line>
│   │   ├── interaction.rs # InteractionParser: Vec<Line> → InteractionSet
│   │   └── participant.rs # ParticipantParser: Vec<Line> → ParticipantSet
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
- Warns (via `log::warn!`) about any lines classified as `LineContents::Invalid`.

### `src/cli.rs` — CLI argument parsing

Three arguments are defined using `clap` 4's builder API:

| Argument | Flag | Notes |
|---|---|---|
| `input` | `-f` / `--file` | Path to a `.seq` input file |
| `example` | `-e` | Use the built-in example diagram (conflicts with `--file`) |
| `output` | (positional) | Required output PNG file path |

Values are read with `options.get_one::<String>(...)` and `options.get_one::<bool>(...)`.

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
| `MetaDataType` | Enum: `Style(String)`, `FontSize(f32)`, `Title(String)`, `Author(String)`, `Date`, `Invalid` |
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

`Theme::default()` uses `Roboto-Thin.ttf` for both title and body fonts. Font loading uses `fontdue::FontSettings { collection_index: 0, scale: 18.0, load_substitutions: true }`.

### `src/parsing/document.rs` — Document parsing

`DocumentParser::parse(input: &[String], config: Config) -> Document`

Uses `Iterator::enumerate` to assign line numbers, and classifies each input line:

- Empty → `LineContents::Empty`
- Starts with `#` → `LineContents::Comment`
- Starts with `:` → metadata via `parse_metadata()`
- Contains `->` → interaction via `parse_interaction()` using the regex `^(.+)\s+-+>+\s+([^:]+):?(.*)$`
- Otherwise → `LineContents::Invalid`

The regex is lazily initialised via `std::sync::OnceLock` (no `lazy_static` dependency).

`parse_metadata` recognises `:theme`, `:title`, `:author`, `:date` keywords. A bare keyword with no trailing value returns `LineContents::Invalid`.

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

- `Diagram::render()` — computes overall image size, creates a `RenderContext` (white background `DrawTarget`), renders all participants sorted by `index`, and writes a PNG via `DrawTarget::write_png()`.
- `Participant::render()` — draws a stroked rectangle (`StrokeStyle { width: 0.5 }`) and the participant label via `draw_text()`.
- `Diagram::size()` (via the `Sizable` trait) — derives `width` and `height` from participant positions and interaction count.
- `RenderContext { pub theme: Theme, pub draw_target: DrawTarget }` — wraps the drawing surface and active theme.

### `src/rendering/text.rs` — Text utilities

- `measure_string(theme, content, px) -> Rect` — uses `fontdue::layout::Layout` to compute the bounding box of a string without drawing it.
- `draw_text(rc, content, x, y, px)` — lays out and rasterises each glyph using `font.rasterize(glyph.parent, px)`, composites it onto the `DrawTarget` via `draw_image_at`.
  - In debug builds, draws a pink bounding box around each glyph (controlled by `#[cfg(debug_assertions)]`).
- `rgb_to_u32(r, g, b, a) -> u32` — packs colour channels into a `u32` pixel value using `((a << 24) | (r << 16) | (g << 8) | b)`.

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
# System dependency (Ubuntu/Debian)
sudo apt-get install -y libfontconfig1-dev

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

Tests are written inside `#[cfg(test)]` modules within each source file. Run them with:

```bash
cargo test
cargo test --verbose
```

### Active tests

**`src/parsing/document.rs`** (`tests` module):

| Test name | What it covers |
|---|---|
| `test_parse_metadata_title` | `:title` metadata parsing |
| `test_parse_metadata_title_with_whitespace` | Metadata parsing with extra whitespace |
| `test_parse_metadata_theme` | `:theme` keyword |
| `test_parse_metadata_author` | `:author` keyword |
| `test_parse_metadata_date` | `:date` keyword |
| `test_parse_metadata_unknown_key` | Unknown metadata key → `MetaDataType::Invalid` |
| `test_parse_metadata_no_value_returns_invalid` | Bare keyword (no value) → `Invalid` |
| `test_parse_interaction_simple` | Simple `A -> B` interaction |
| `test_parse_interaction_with_message` | `A -> B: msg` interaction |
| `test_parse_interaction_no_match` | Non-interaction line → `Invalid` |
| `test_document_parser_with_invalid` | Mixed valid/invalid lines produce correct `LineContents` |
| `test_document_parser` | Full document with metadata, interactions, empty lines |
| `test_document_parser_comment_lines` | Comment lines produce `LineContents::Comment` |
| `test_document_is_valid` | `Document.is_valid` is `true` for a valid document |

**`src/rendering/text.rs`** (`tests` module):

| Test name | What it covers |
|---|---|
| `test_rgb_to_u32_black` | Packs black (0,0,0,255) correctly |
| `test_rgb_to_u32_white` | Packs white (255,255,255,255) correctly |
| `test_rgb_to_u32_blue` | Packs blue (0,0,255,255) correctly |
| `test_rgb_to_u32_transparent` | Packs transparent (0,0,0,0) correctly |
| `test_rgb_to_u32_clamps_above_255` | Values >255 are clamped |
| `test_measure_string_single_char` | Single char returns sensible `Rect` |
| `test_measure_string_two_chars_wider` | Two chars produce wider `Rect` than one |
| `test_measure_string_same_height_for_same_font_size` | Same px → same height |
| `test_measure_string_larger_px_gives_larger_height` | Larger px → taller glyphs |

### Commented-out tests

Several tests in `src/parsing/interaction.rs` and `src/parsing/participant.rs` are commented out because `Participant` now carries a `rect: Rect` field that requires `Theme` to compute. These tests can be re-enabled by constructing a `Theme::default()` and passing it to `ParticipantParser::parse`.

---

## Benchmarks

All four benchmark suites are active. Run them with:

```bash
cargo bench
```

| File | Benchmark name | What it measures |
|---|---|---|
| `benches/benchmark_document_parsing.rs` | `parsing document` | `DocumentParser::parse` throughput |
| `benches/benchmark_diagram_parsing.rs` | `parsing participants` | `ParticipantParser::parse` throughput |
| `benches/benchmark_diagram_parsing.rs` | `parsing interactions` | `InteractionParser::parse` throughput |
| `benches/benchmark_rendering.rs` | `diagram parse` | Full `DocumentParser::parse` + `Diagram::parse` cycle |
| `benches/text_benchmarks.rs` | `measure_string single char` | `measure_string` for one character |
| `benches/text_benchmarks.rs` | `measure_string long string` | `measure_string` for a longer string |
| `benches/text_benchmarks.rs` | `rgb_to_u32` | Pixel packing throughput |

---

## CI / CD

### `.github/workflows/build_and_test.yml`

Runs on every push and pull request to `main`:
1. Install `libfontconfig1-dev`
2. `cargo build --verbose`
3. `cargo test --verbose`

### `.github/workflows/codecov.yml`

Runs on every push and pull request to `main`:
1. Install `libfontconfig1-dev`
2. Install `cargo-tarpaulin`
3. `cargo tarpaulin --out Xml` — collects line coverage
4. Uploads results to [Codecov](https://codecov.io/gh/rsouth/seq-rs) and as a workflow artifact

---

## Key Design Decisions and Notes

- **Library + binary split**: The crate is structured as a library (`lib.rs`) consumed by the binary (`main.rs`). This enables benchmarks and future integration tests to import public types and parsers directly.
- **Fonts are embedded at compile time**: `include_bytes!` bakes the TTF files into the binary so no runtime font discovery is needed.
- **`HashSet<Participant>` for participants**: Participants are stored in a `HashSet` and sorted by `index` at render time. This avoids duplicates when a participant appears in multiple interactions.
- **Single-pass participant discovery**: `ParticipantParser` assigns indices in order of first appearance in the document, which determines left-to-right visual ordering.
- **`raqote` + `fontdue` for rendering**: `raqote` provides a software 2D canvas (no GPU required); `fontdue` handles font loading, layout, and glyph rasterisation.
- **`OnceLock` for regex**: The interaction regex is initialised once via `std::sync::OnceLock` — no `lazy_static` dependency.
- **`fontdue` 0.9 API**: Uses `font.rasterize(glyph.parent, px)` where `glyph.parent` is the source `char`. The old `rasterize_indexed` API is no longer used.
- **Interactions not yet fully differentiated**: Arrow variants (`->`, `-->`, `->>`, `-->>`) are all parsed the same way by the current regex; the dashed/async distinction is not yet surfaced in the model.
- **`LineContents::Invalid` is non-fatal**: Invalid lines produce a `warn!` log but do not stop diagram generation.
- **`Cargo.lock` is gitignored**: The project is a binary crate. Per Rust best practices, binary projects should commit `Cargo.lock` to guarantee reproducible builds. The current `.gitignore` excludes it; consider removing that exclusion.
- **`src/mod.rs` is a legacy file**: It exists but contains only commented-out code; it is not imported or used anywhere.

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
