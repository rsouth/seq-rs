# AGENTS.md

This file is a working guide for people and coding agents making changes in `rsouth/seq-rs`.

## Project summary

`seq-rs` is a Rust sequence-diagram generator. The repository builds the `sequencer` crate/binary, which parses a lightweight text DSL and writes a PNG diagram to disk.

Core responsibilities:

- parse line-oriented diagram input
- discover participants and interactions
- compute simple diagram layout
- render diagram output with embedded fonts

The current implementation is intentionally compact, so most behavior is easy to trace from `main.rs` into a handful of parser and rendering modules.

## High-level architecture

The main execution path is:

1. `src/cli.rs`
   - defines the CLI with `clap`
   - supports `--file`, `-e/--example`, and a required output path
2. `src/main.rs`
   - resolves the input source
   - loads raw text from a file, stdin, or the built-in example
   - builds `Config`
   - runs `DocumentParser::parse`
   - creates `Theme::default()`
   - calls `Diagram::parse(...)`
   - writes the final PNG with `diagram.render()`
3. `src/parsing/document.rs`
   - classifies each line as empty, comment, metadata, interaction, or invalid
   - stores parsed lines in `Document`
4. `src/parsing/participant.rs`
   - finds all participants referenced by interactions
   - assigns participant indexes in first-seen order
   - computes x positions and participant rectangles using measured text width
5. `src/parsing/interaction.rs`
   - turns parsed interaction lines into `Interaction` values
   - derives interaction direction from participant indexes
6. `src/rendering/`
   - `mod.rs` creates the drawing surface and renders participants
   - `text.rs` measures and rasterizes text with `fontdue`
7. `src/theme.rs`
   - embeds fonts from `assets/`
   - stores layout constants such as font sizes, padding, and gaps

## Important files and directories

### Root

- `/home/runner/work/seq-rs/seq-rs/Cargo.toml`  
  Rust package manifest, dependencies, and benchmark declarations.
- `/home/runner/work/seq-rs/seq-rs/README.md`  
  Public project overview and quick-start documentation.
- `/home/runner/work/seq-rs/seq-rs/AGENTS.md`  
  This contributor/agent guide.

### Source code

- `/home/runner/work/seq-rs/seq-rs/src/main.rs`  
  Runtime entry point and built-in example DSL.
- `/home/runner/work/seq-rs/seq-rs/src/cli.rs`  
  CLI definitions.
- `/home/runner/work/seq-rs/seq-rs/src/lib.rs`  
  Public module declarations and shared type aliases.
- `/home/runner/work/seq-rs/seq-rs/src/model.rs`  
  Shared domain types like `Line`, `Participant`, `Interaction`, and `Config`.
- `/home/runner/work/seq-rs/seq-rs/src/diagram.rs`  
  Converts a parsed `Document` into a `Diagram`.
- `/home/runner/work/seq-rs/seq-rs/src/parsing/document.rs`  
  First-pass parser from raw strings into `LineContents`.
- `/home/runner/work/seq-rs/seq-rs/src/parsing/participant.rs`  
  Participant discovery and layout preparation.
- `/home/runner/work/seq-rs/seq-rs/src/parsing/interaction.rs`  
  Interaction construction and direction detection.
- `/home/runner/work/seq-rs/seq-rs/src/rendering/mod.rs`  
  Draw target setup, sizing, and participant rendering.
- `/home/runner/work/seq-rs/seq-rs/src/rendering/text.rs`  
  Text measurement and glyph drawing.
- `/home/runner/work/seq-rs/seq-rs/src/theme.rs`  
  Default font loading and visual spacing.

### Other directories

- `/home/runner/work/seq-rs/seq-rs/assets/`  
  Bundled fonts used by the renderer.
- `/home/runner/work/seq-rs/seq-rs/benches/`  
  Criterion benchmarks for parsing and rendering.
- `/home/runner/work/seq-rs/seq-rs/docs/`  
  Checked-in documentation assets, including the example output PNG used in `README.md`.
- `/home/runner/work/seq-rs/seq-rs/.github/workflows/build_and_test.yml`  
  Canonical CI build/test steps.

## Data model quick reference

Important types in `src/model.rs`:

- `Config` — selected input source and output path
- `Source` — stdin, file, or built-in example
- `Line` — original line text plus parsed classification
- `LineContents` — parsed line variants
- `Participant` — participant name, index, activity range, and layout rectangle
- `Interaction` — from/to participants, direction, optional message
- `InteractionType` — left-to-right, right-to-left, or self-reference

## Current parser and renderer behavior

### Document parser

The document parser is line-based and intentionally permissive:

- empty lines become `LineContents::Empty`
- `#` lines become comments
- `:` lines are parsed as metadata
- lines containing `->` are parsed as interactions
- anything else becomes `Invalid`

Metadata currently recognizes:

- `:theme`
- `:title`
- `:author`
- `:date`

### Participant parser

The participant parser walks interaction lines only. It:

- records first-seen order for stable participant indexes
- tracks first and last interaction positions
- measures participant names to compute rectangles
- shares a common max height across participant boxes

### Interaction parser

The interaction parser:

- converts parsed interaction lines into `Interaction` structs
- looks up participants by name
- sets `InteractionType` from participant index ordering
- preserves optional message text

### Rendering

The renderer currently:

- computes a drawing surface size from participant bounds and interaction count
- draws participant boxes
- draws participant text
- writes a PNG to the requested output path

This repository already has strong unit coverage for parsing and text helpers. The rendering surface is small enough that visual verification is still useful for documentation or layout work.

## Commands you should use

### Prerequisite on Ubuntu/Linux

The Rust dependencies require fontconfig development headers:

```bash
sudo apt-get update
sudo apt-get install -y libfontconfig1-dev
```

### Validate the current code

```bash
cd /home/runner/work/seq-rs/seq-rs
cargo build
cargo test
```

### Generate the built-in example output

```bash
cd /home/runner/work/seq-rs/seq-rs
cargo run -- -e docs/example-output.png
```

### Run benchmarks

```bash
cd /home/runner/work/seq-rs/seq-rs
cargo bench
```

## How to make safe changes here

1. Keep changes narrow. Most features are localized to one parser or one rendering module.
2. Prefer updating existing modules rather than introducing new abstraction layers.
3. If you change parsing behavior, add or adjust unit tests in the same parser module.
4. If you change rendering or layout behavior, run `cargo test` and also generate a real PNG for manual inspection.
5. Avoid changing embedded fonts or theme defaults unless the task specifically requires it.

## Common gotchas

- `cargo test` and `cargo build` can fail on clean Ubuntu systems without `libfontconfig1-dev`.
- The output path is required by the CLI, even when using the built-in example.
- The built-in example input lives in `src/main.rs` (`get_text()`), so README screenshots should match that source when possible.
- The rendering code is compact and direct; visual regressions are easiest to catch by generating an actual output PNG.

## Recommended workflow for future agents

When working on this repository:

1. Read `README.md` and the relevant source module first.
2. Confirm the native dependency is installed before assuming Rust failures are code regressions.
3. Run existing tests before editing code.
4. Make the smallest possible change in the parser or renderer that solves the task.
5. Run targeted validation quickly, then run `cargo test`.
6. If the change affects output or layout, generate a PNG and inspect it manually.

## If you need a quick orientation

Start here in order:

1. `/home/runner/work/seq-rs/seq-rs/src/main.rs`
2. `/home/runner/work/seq-rs/seq-rs/src/parsing/document.rs`
3. `/home/runner/work/seq-rs/seq-rs/src/parsing/participant.rs`
4. `/home/runner/work/seq-rs/seq-rs/src/parsing/interaction.rs`
5. `/home/runner/work/seq-rs/seq-rs/src/rendering/mod.rs`
6. `/home/runner/work/seq-rs/seq-rs/src/rendering/text.rs`
