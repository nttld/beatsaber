use std::path::Path;

use anyhow::Result;
use beatsaber::codegen::{self, CodegenOptions};
use beatsaber::{ast1, ast2, lexer};
use clap::{AppSettings, Clap};
use std::fs;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "untitled")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Args {
    /// Input source file path.
    input: String,
    /// Output object file path.
    #[clap(short)]
    output: Option<String>,
    /// Target triple
    #[clap(long)]
    target: Option<String>,
    /// Optimization level
    #[clap(short = 'O', default_value = "2")]
    optimization: u8,
    /// Generate position independent code
    #[clap(long)]
    pic: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let output_path = args.output.clone().unwrap_or_else(|| {
        let input_path = Path::new(&args.input);
        input_path
            .with_extension("o")
            .into_os_string()
            .into_string()
            .unwrap()
    });
    let output_path = Path::new(&output_path);

    let src = fs::read_to_string(&args.input).unwrap();
    let lexer = lexer::lexer(&src, &args.input);
    let parser = ast1::parser(lexer);
    let ast2 = ast2::parse(parser);
    let options = CodegenOptions {
        output: Path::new(output_path),
        optimization: codegen::OptLevel::Aggressive,
        pic: true,
        target: None,
    };
    dbg!(&ast2);
    codegen::Codegen::compile(ast2, options)?;

    Ok(())
}
