use std::{error::Error, fs};

use clap::Parser;
use pop65::assemble;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let (bytes, symtab) = assemble(pop65::from_file(&cli.source)?)?;
    fs::write(cli.output, bytes)?;
    if let Some(sympath) = cli.symbols {
        let mut symstr = String::new();
        let mut symbols = Vec::from_iter(symtab.values());
        symbols.sort();
        for symbol in symbols {
            symstr.push_str(&format!("{}\n", symbol));
        }
        fs::write(sympath, symstr)?;
    }
    Ok(())
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    source: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    symbols: Option<String>,
}
