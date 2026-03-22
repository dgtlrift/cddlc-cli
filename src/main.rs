mod args;
mod loader;

use std::path::PathBuf;
use std::process;

use clap::Parser;

use args::Cli;
use cddlc_codegen::{Backend, CodegenOptions};
use cddlc_ir::lower;
use loader::load;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    // ── Validate flags ────────────────────────────────────────────────────────

    if cli.no_std && cli.lang != args::Lang::Rust {
        return Err("--no-std is only valid with --lang rust".into());
    }

    // ── Load and merge all input files ────────────────────────────────────────

    if cli.verbose {
        eprintln!("cddlc: loading {} input file(s)...", cli.inputs.len());
    }

    // For multiple inputs, load each and concatenate rules
    let mut merged_rules  = Vec::new();
    let mut merged_pragmas = Vec::new();
    let mut all_warnings  = Vec::new();
    let mut primary_path  = PathBuf::from("module.cddl");

    for (i, input) in cli.inputs.iter().enumerate() {
        let loaded = load(input, &cli.include_dir, cli.verbose)
            .map_err(|e| e.to_string())?;

        all_warnings.extend(loaded.warnings);

        if i == 0 {
            primary_path = loaded.module.source_path.clone();
            merged_pragmas = loaded.module.file_pragmas;
        } else {
            // Additional inputs: collect only their @import pragmas
            merged_pragmas.extend(
                loaded.module.file_pragmas.into_iter()
                    .filter(|p| p.keyword == "import")
            );
        }

        merged_rules.extend(loaded.module.rules);
    }

    // Print parse warnings
    for w in &all_warnings {
        eprintln!("warning: {w}");
    }

    let merged_module = cddlc_parser::CddlModule {
        source_path: primary_path,
        file_pragmas: merged_pragmas,
        rules: merged_rules,
    };

    // ── Semantic analysis (IR lowering) ───────────────────────────────────────

    if cli.verbose {
        eprintln!("cddlc: running semantic analysis...");
    }

    let lower_result = lower(&merged_module, cli.max_array, cli.max_str)
        .map_err(|e| e.to_string())?;

    for w in &lower_result.warnings {
        eprintln!("warning: {w}");
    }

    let ir = &lower_result.module;

    if cli.verbose {
        eprintln!("cddlc: {} type(s) resolved:", ir.types.len());
        for (name, def) in &ir.types {
            eprintln!("  {} {:?}", name, std::mem::discriminant(def));
        }
    }

    // ── Dry run — stop before codegen ─────────────────────────────────────────

    if cli.dry_run {
        eprintln!("cddlc: dry-run complete ({} types, {} warnings)",
            ir.types.len(),
            all_warnings.len() + lower_result.warnings.len());
        return Ok(());
    }

    // ── Select backend ────────────────────────────────────────────────────────

    let opts = CodegenOptions {
        lang:        cli.lang.into(),
        runtime:     cli.runtime.clone(),
        alloc:       cli.alloc.into(),
        dcbor:       cli.dcbor,
        no_std:      cli.no_std,
        depth_limit: cli.depth_limit,
        namespace:   cli.namespace.clone(),
        max_array:   cli.max_array,
        max_str:     cli.max_str,
    };

    let output = match cli.lang {
        args::Lang::Rust => {
            let backend = backend_rust::RustBackend;
            backend.generate(ir, &opts)
                .map_err(|e| e.to_string())?
        }
        args::Lang::C => {
            let backend = backend_c::CBackend;
            backend.generate(ir, &opts)
                .map_err(|e| e.to_string())?
        }
        args::Lang::Cpp => {
            let backend = backend_cpp::CppBackend;
            backend.generate(ir, &opts)
                .map_err(|e| e.to_string())?
        }
        args::Lang::Csharp => {
            let backend = backend_csharp::CSharpBackend;
            backend.generate(ir, &opts)
                .map_err(|e| e.to_string())?
        }
        args::Lang::Nodejs => {
            let backend = backend_nodejs::NodeJsBackend;
            backend.generate(ir, &opts)
                .map_err(|e| e.to_string())?
        }
        args::Lang::Python => {
            let backend = backend_python::PythonBackend;
            backend.generate(ir, &opts)
                .map_err(|e| e.to_string())?
        }
    };

    // ── Write output files ────────────────────────────────────────────────────

    std::fs::create_dir_all(&cli.output)?;

    for file in &output.files {
        let dest = cli.output.join(&file.path);

        // Create subdirectories if needed
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&dest, &file.content)?;

        if cli.verbose || true {  // always print generated files
            eprintln!("cddlc: wrote {}", dest.display());
        }
    }

    Ok(())
}
