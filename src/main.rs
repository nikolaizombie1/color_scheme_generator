use anyhow::Result;
use color_scheme_generator::theme_calculation;
use std::{
   io::{stdin, IsTerminal, Read}, path::PathBuf
};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true, index = 1, value_parser = is_image)]
    image: PathBuf,
    #[arg(short, long, required = true)]
    centrality: theme_calculation::Centrality,
    #[arg(short, long, default_value_t = 5)]
    number_of_themes: u8
}

fn is_image(input: &str) -> anyhow::Result<PathBuf> {
    image::io::Reader::open(input)?.with_guessed_format()?.format().ok_or(std::fmt::Error)?;
    Ok(input.parse::<PathBuf>()?)
}

fn main() -> Result<()> {
    let args = if stdin().is_terminal() {
        Args::parse()
    } else {
        let mut input = String::new();
        let mut stdin = stdin().lock();
        while let Ok(x) = stdin.read_to_string(&mut input) {
            if x == 0 {
                break;
            }
        }
	let input = String::from(input.trim());
        let mut args = std::env::args().collect::<Vec<_>>();
	args.push(input);
        Args::parse_from(args.iter())
    };
    println!("{}", args.image.as_path().to_str().unwrap());
    Ok(())
}
