#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use color_scheme_generator::{database, theme_calculation};
use std::{
    io::{stdin, IsTerminal, Read}, path::PathBuf
};

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true, index = 1, value_parser = is_image)]
    image: PathBuf,
    #[arg(short, long, default_value_t = theme_calculation::Centrality::Prevalent)]
    centrality: theme_calculation::Centrality,
    #[arg(short, long, default_value_t = 5)]
    number_of_themes: u8,
    #[arg(short, long, default_value_t = OutputFormat::JSON)]
    serialization_format: OutputFormat,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    JSON,
    YAML,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::JSON => write!(f, "{}", "json"),
            OutputFormat::YAML => write!(f, "{}", "yaml"),
        }
    }
}

const APP_NAME: &str = "color_scheme_generator";

fn is_image(input: &str) -> anyhow::Result<PathBuf> {
    let path = input.parse::<PathBuf>()?;
    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME)?;
    let cache_path = xdg_dirs.place_cache_file("cache.db")?;
    let conn = database::DatabaseConnection::new(&cache_path)?;
    match conn.select_color_theme_by_image_path(&path) {
        Ok(_) => Ok(path),
        Err(_) => {
            image::io::Reader::open(input)?
                .with_guessed_format()?
                .format()
                .ok_or(std::fmt::Error)?;
            Ok(path)
        }
    }
}

fn main() -> anyhow::Result<()> {
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

    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME)?;
    let cache_path = xdg_dirs.place_cache_file("cache.db")?;
    let conn = database::DatabaseConnection::new(&cache_path)?;
    let color_themes: Vec<theme_calculation::ColorTheme> =
        match conn.select_color_theme_by_image_path(&args.image) {
            Ok(c) => c,
            Err(_) => {
                let theme = theme_calculation::generate_color_theme(
                    &args.image,
                    args.centrality,
                    args.number_of_themes,
                );
                conn.insert_color_theme_record(&args.image, &theme)?;
                theme
            }
        };

    let output: String = match args.serialization_format {
        OutputFormat::JSON => serde_json::to_string::<Vec<theme_calculation::ColorTheme>>(&color_themes)?,
        OutputFormat::YAML => serde_yml::to_string::<Vec<theme_calculation::ColorTheme>>(&color_themes)?,
    };
    println!("{}", output);
    Ok(())
}
