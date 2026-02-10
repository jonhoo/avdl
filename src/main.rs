// ==============================================================================
// CLI for the Avro IDL Parser
// ==============================================================================
//
// Two subcommands that mirror the Java `avro-tools` interface:
//   - `avdl idl [INPUT] [OUTPUT]`        -- compile .avdl to .avpr or .avsc JSON
//   - `avdl idl2schemata [INPUT] [OUTDIR]` -- extract individual .avsc files

use std::fs;
use std::io::{self, Read as _};
use std::path::PathBuf;

use avdl::{Idl, Idl2Schemata};
use lexopt::prelude::*;

// ==============================================================================
// CLI Help Text
// ==============================================================================

const MAIN_HELP: &str = "\
avdl - Avro IDL compiler

Usage: avdl <COMMAND>

Commands:
  idl           Compile an Avro IDL file to protocol (.avpr) or schema (.avsc) JSON
  idl2schemata  Extract individual .avsc schema files from an Avro IDL protocol

Options:
  -h, --help    Print help";

const IDL_HELP: &str = "\
Usage: avdl idl [OPTIONS] [INPUT] [OUTPUT]

Options:
      --import-dir <DIR>  Additional directories to search for imports (repeatable)
  -h, --help              Print help";

const IDL2SCHEMATA_HELP: &str = "\
Usage: avdl idl2schemata [OPTIONS] INPUT [OUTDIR]

Options:
      --import-dir <DIR>  Additional directories to search for imports (repeatable)
  -h, --help              Print help";

// ==============================================================================
// Argument Parsing
// ==============================================================================

/// Parsed CLI arguments for the `idl` subcommand.
struct IdlArgs {
    input: Option<String>,
    output: Option<String>,
    import_dirs: Vec<PathBuf>,
}

/// Parsed CLI arguments for the `idl2schemata` subcommand.
struct Idl2schemataArgs {
    input: String,
    outdir: Option<PathBuf>,
    import_dirs: Vec<PathBuf>,
}

/// Parse `--import-dir` and positional args for the `idl` subcommand.
fn parse_idl_args(parser: &mut lexopt::Parser) -> Result<IdlArgs, lexopt::Error> {
    let mut import_dirs = Vec::new();
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("import-dir") => {
                let val: String = parser.value()?.string()?;
                import_dirs.push(PathBuf::from(val));
            }
            Short('h') | Long("help") => {
                println!("{IDL_HELP}");
                std::process::exit(0);
            }
            Value(val) => {
                positionals.push(val.string()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    let input = positionals.first().cloned();
    let output = positionals.get(1).cloned();

    Ok(IdlArgs {
        input,
        output,
        import_dirs,
    })
}

/// Parse `--import-dir` and positional args for the `idl2schemata` subcommand.
fn parse_idl2schemata_args(parser: &mut lexopt::Parser) -> Result<Idl2schemataArgs, lexopt::Error> {
    let mut import_dirs = Vec::new();
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("import-dir") => {
                let val: String = parser.value()?.string()?;
                import_dirs.push(PathBuf::from(val));
            }
            Short('h') | Long("help") => {
                println!("{IDL2SCHEMATA_HELP}");
                std::process::exit(0);
            }
            Value(val) => {
                positionals.push(val.string()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    let input = positionals
        .first()
        .cloned()
        .ok_or_else(|| lexopt::Error::MissingValue {
            option: Some("INPUT".to_string()),
        })?;
    let outdir = positionals.get(1).map(PathBuf::from);

    Ok(Idl2schemataArgs {
        input,
        outdir,
        import_dirs,
    })
}

// ==============================================================================
// Entry Point
// ==============================================================================

fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))?;

    let mut parser = lexopt::Parser::from_env();

    // The first positional value is the subcommand name.
    let subcommand = match parser.next() {
        Ok(Some(Value(val))) => val.string().map_err(|e| miette::miette!("{e}"))?,
        Ok(Some(Short('h') | Long("help"))) => {
            println!("{MAIN_HELP}");
            return Ok(());
        }
        Ok(Some(other)) => {
            let err = other.unexpected();
            eprintln!("error: {err}\n\n{MAIN_HELP}");
            std::process::exit(2);
        }
        Ok(None) => {
            eprintln!("error: a subcommand is required\n\n{MAIN_HELP}");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    match subcommand.as_str() {
        "idl" => {
            let args = parse_idl_args(&mut parser).map_err(|e| miette::miette!("{e}"))?;
            run_idl(args.input, args.output, args.import_dirs)
        }
        "idl2schemata" => {
            let args = parse_idl2schemata_args(&mut parser).map_err(|e| miette::miette!("{e}"))?;
            run_idl2schemata(args.input, args.outdir, args.import_dirs)
        }
        other => {
            eprintln!("error: unknown subcommand `{other}`\n\n{MAIN_HELP}");
            std::process::exit(2);
        }
    }
}

// ==============================================================================
// `idl` Subcommand
// ==============================================================================

fn run_idl(
    input: Option<String>,
    output: Option<String>,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<()> {
    let mut builder = Idl::new();
    for dir in &import_dirs {
        builder.import_dir(dir);
    }

    let idl_output = match &input {
        Some(path) if path != "-" => builder.convert(path)?,
        _ => {
            // Read from stdin.
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .map_err(|e| miette::miette!("{e}: read IDL from stdin"))?;
            let source_name = input.as_deref().unwrap_or("<stdin>");
            builder.convert_str_named(&source, source_name)?
        }
    };

    // Emit any warnings to stderr. When source context is available, render
    // through miette for rich output with source underlining. Otherwise, fall
    // back to plain text (e.g., import-prefixed warnings where source was cleared).
    for w in &idl_output.warnings {
        eprintln!("{w:?}");
    }

    let json_str = serde_json::to_string_pretty(&idl_output.json)
        .map_err(|e| miette::miette!("serialize JSON: {e}"))?;

    write_output(&output, &json_str)?;

    Ok(())
}

// ==============================================================================
// `idl2schemata` Subcommand
// ==============================================================================

fn run_idl2schemata(
    input: String,
    outdir: Option<PathBuf>,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<()> {
    let mut builder = Idl2Schemata::new();
    for dir in &import_dirs {
        builder.import_dir(dir);
    }

    let schemata_output = builder.extract(&input)?;

    // Emit any warnings to stderr. When source context is available, render
    // through miette for rich output with source underlining. Otherwise, fall
    // back to plain text (e.g., import-prefixed warnings where source was cleared).
    for w in &schemata_output.warnings {
        eprintln!("{w:?}");
    }

    let output_dir = outdir.unwrap_or_else(|| PathBuf::from("."));
    if output_dir.exists() && !output_dir.is_dir() {
        return Err(miette::miette!(
            "output path `{}` exists and is not a directory",
            output_dir.display()
        ));
    }
    fs::create_dir_all(&output_dir).map_err(|e| miette::miette!("{e}: create output directory"))?;

    for named_schema in &schemata_output.schemas {
        let json_str = serde_json::to_string_pretty(&named_schema.schema)
            .map_err(|e| miette::miette!("serialize JSON for {}: {e}", named_schema.name))?;

        let file_path = output_dir.join(format!("{}.avsc", named_schema.name));
        // Append trailing newline to match Java's `PrintStream.println()`.
        fs::write(&file_path, format!("{json_str}\n"))
            .map_err(|e| miette::miette!("{e}: write {}", file_path.display()))?;
    }

    Ok(())
}

// ==============================================================================
// Output Writing
// ==============================================================================

/// Write output to a file or stdout.
fn write_output(output: &Option<String>, content: &str) -> miette::Result<()> {
    // Treat `None` and `Some("-")` as stdout; everything else is a file path.
    let file_path = output.as_deref().filter(|s| *s != "-");

    match file_path {
        None => {
            // Write to stdout without trailing newline, matching Java behavior.
            // Handle BrokenPipe gracefully.
            use std::io::Write;
            if let Err(e) = write!(io::stdout(), "{content}") {
                if e.kind() == io::ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(miette::miette!("{e}: write to stdout"));
            }
            Ok(())
        }
        Some(file_path) => {
            let path = PathBuf::from(file_path);
            // Append a trailing newline to match the golden files.
            fs::write(&path, format!("{content}\n"))
                .map_err(|e| miette::miette!("{e}: write {}", path.display()))
        }
    }
}
