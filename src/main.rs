#![deny(unused_extern_crates)]
#![warn(missing_docs)]

//! Quickly generate color schemes for waybar from an image.
//!
//! color_scheme_generator is a command line utility used to analyze images
//! and generate color themes from them given a path to an image.
//!
//! This command line utility behaves like a standard UNIX utility where the path to the image can be either piped in or sent a command line argument.
//!
//! The intended purpose of this application is to automatically create color themes for
//! Waybar, but it used for the bar in AwesomeWM or other applications to theme based on the on an image.
//! This utility has a cache for the image analysis. This means that once an image has been analyzed once, the result will be saved in the cache and when an image is analyzed again, the results will be returned instantly.
//!
//! # Usage Examples
//! ```bash
//! echo PATH_TO_IMAGE | color_scheme_generator
//! ```
//! ```bash
//! color_scheme_generator PATH_TO_IMAGE
//! ```
//!
//! # Output Formats
//! color_scheme_generator can output to 3 different output formats all of which give an RGB8 value in the form of "bar_color", "workspace_color" and "text_color":
//! 1. JSON
//! ```json
//! [{"bar_color":{"red":222,"green":186,"blue":189},"workspace_color":{"red":33,"green":69,"blue":66},"text_color":{"red":255,"green":255,"blue":255}}]
//! ```
//! 2. YAML
//! ```yaml
//! - bar_color:
//!     red: 222
//!     green: 186
//!     blue: 189
//!   workspace_color:
//!     red: 33
//!     green: 69
//!     blue: 66
//!   text_color:
//!     red: 255
//!     green: 255
//!     blue: 255
//! ```
//! 3. Text
//! ```bash
//! DEBABD,214542,FFFFFF
//! ```
//! The text output has the format of `BAR_COLOR,WORKSPACE_COLOR,TEXT_COLOR`.

use color_scheme_generator::{database, theme_calculation};
use std::{
    io::{stdin, IsTerminal, Read},
    path::PathBuf,
};

use clap::{Parser, ValueEnum};

/// Command line argument Struct used by clap to parse CLI arguments.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the image file.
    #[arg(required = true, index = 1, value_parser = is_image)]
    image: PathBuf,
    /// Measure of centrality to be used to analyze an image.
    #[arg(short, long, default_value_t = theme_calculation::Centrality::Prevalent)]
    centrality: theme_calculation::Centrality,
    /// Number of color themes to be generated. (Only works in prevalent centrality).
    #[arg(short, long, default_value_t = 1)]
    number_of_themes: u8,
    /// Output format for color themes.
    #[arg(short, long, default_value_t = OutputFormat::JSON)]
    serialization_format: OutputFormat,
}

/// Output format for [`color_scheme_generator::theme_calculation::ColorTheme`].
#[derive(Clone, ValueEnum)]
enum OutputFormat {
    JSON,
    YAML,
    TEXT,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::JSON => write!(f, "{}", "json"),
            OutputFormat::YAML => write!(f, "{}", "yaml"),
            OutputFormat::TEXT => write!(f, "{}", "text"),
        }
    }
}

/// Application Name used for XDG compliant directory structure.
const APP_NAME: &str = "color_scheme_generator";

/// Custom validator for [`Args::image`].
///
/// First checks if image path is in the cache, if not, checks if the image file is valid.
fn is_image(input: &str) -> anyhow::Result<PathBuf> {
    let path = input.parse::<PathBuf>()?;
    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME)?;
    let cache_path = xdg_dirs.place_cache_file("cache.db")?;
    let conn = database::DatabaseConnection::new(&cache_path)?;
    match conn.select_color_theme_by_image_path(&path) {
        Ok(_) => Ok(path),
        Err(_) => {
            image::ImageReader::open(input)?
                .with_guessed_format()?
                .format()
                .ok_or(std::fmt::Error)?;
            Ok(path)
        }
    }
}

/// Starting point of the application.
///
/// Check if program is in pipe, if so receive stdin and parse arguments and stdin.
/// Else, parse the arguments normally.
///
/// Creates cache inside of XDG_CACHE_HOME,
/// check if image is in cache, if so return theme,
/// else analyze the image and add it to cache.
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
                )?;
                conn.insert_color_theme_record(&args.image, &theme)?;
                theme
            }
        };

    let output: String = match args.serialization_format {
        OutputFormat::JSON => {
            serde_json::to_string::<Vec<theme_calculation::ColorTheme>>(&color_themes)?
        }
        OutputFormat::YAML => {
            serde_yml::to_string::<Vec<theme_calculation::ColorTheme>>(&color_themes)?
        }
        OutputFormat::TEXT => {
            let x = color_themes
                .iter()
                .map(|c| {
                    format!(
                        "{:02x?}{:02x?}{:02x?},{:02x?}{:02x?}{:02x?},{:02x?}{:02x?}{:02x?}",
                        c.bar_color.red,
                        c.bar_color.green,
                        c.bar_color.blue,
                        c.workspace_color.red,
                        c.workspace_color.green,
                        c.workspace_color.blue,
                        c.text_color.red,
                        c.text_color.green,
                        c.text_color.blue
                    )
                })
                .collect::<Vec<_>>();
            let x = x.iter().map(|s| s.to_ascii_uppercase()).collect::<Vec<_>>();
            let mut ret = String::new();
            for c in x {
                ret += &String::from(c + "\n");
            }
            ret
        }
    };
    println!("{}", output);
    Ok(())
}
