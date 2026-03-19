use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// cddlc — CDDL compiler and code generator
///
/// Parses RFC 8610 CDDL schema files and generates serializers/deserializers
/// for the specified target language.
#[derive(Parser, Debug)]
#[command(name = "cddlc", version, about, long_about = None)]
pub struct Cli {
    /// One or more .cddl source files (entry points)
    #[arg(required = true, value_name = "INPUT")]
    pub inputs: Vec<PathBuf>,

    /// Target language
    #[arg(short, long, value_enum, default_value = "rust")]
    pub lang: Lang,

    /// Output directory for generated files
    #[arg(short, long, default_value = "./generated")]
    pub output: PathBuf,

    /// CBOR runtime library
    ///   rust: minicbor | ciborium | cbor4ii
    ///   c:    tinycbor | nanocbor | zcbor
    #[arg(short, long, default_value = "minicbor")]
    pub runtime: String,

    /// Emit deterministic encoding (dCBOR, RFC 8949 §4.2)
    #[arg(long)]
    pub dcbor: bool,

    /// Allocation strategy for generated code
    #[arg(long, value_enum, default_value = "arena")]
    pub alloc: Alloc,

    /// Rust: emit #![no_std] + heapless collections (implies arena/stack)
    #[arg(long)]
    pub no_std: bool,

    /// Maximum nesting depth for decoder
    #[arg(long, default_value = "16")]
    pub depth_limit: usize,

    /// Default capacity for unbounded sequences (e.g. `[* T]`)
    #[arg(long, default_value = "16", value_name = "N")]
    pub max_array: usize,

    /// Default capacity for unbounded strings
    #[arg(long, default_value = "64", value_name = "N")]
    pub max_str: usize,

    /// Namespace / module prefix for emitted symbols
    #[arg(long, value_name = "NS")]
    pub namespace: Option<String>,

    /// Additional search paths for imported .cddl files (repeatable)
    #[arg(long, value_name = "DIR")]
    pub include_dir: Vec<PathBuf>,

    /// Parse and analyze only — do not write output files
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose output (print resolved types, warnings, file paths)
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    C,
    Cpp,
    Csharp,
    Python,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alloc {
    Stack,
    Arena,
    Heap,
}

impl From<Lang> for cddlc_codegen::Language {
    fn from(l: Lang) -> Self {
        match l {
            Lang::Rust   => cddlc_codegen::Language::Rust,
            Lang::C      => cddlc_codegen::Language::C,
            Lang::Cpp    => cddlc_codegen::Language::Cpp,
            Lang::Csharp => cddlc_codegen::Language::CSharp,
            Lang::Python => cddlc_codegen::Language::Python,
        }
    }
}

impl From<Alloc> for cddlc_codegen::AllocStrategy {
    fn from(a: Alloc) -> Self {
        match a {
            Alloc::Stack => cddlc_codegen::AllocStrategy::Stack,
            Alloc::Arena => cddlc_codegen::AllocStrategy::Arena,
            Alloc::Heap  => cddlc_codegen::AllocStrategy::Heap,
        }
    }
}
