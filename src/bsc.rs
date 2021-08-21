use std::path::Path;

use clap::{AppSettings, Clap};
use anyhow::Result;
use std::fs;
use beatsaber::{ast1, lexer};

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
    let lexer = lexer::lexer(&src);
    let ast = ast1::parse(lexer);

    dbg!(ast);

    Ok(())
}