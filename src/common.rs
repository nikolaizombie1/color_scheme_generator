use anyhow;
use clap::{Args, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Error}, path::PathBuf, str::FromStr
};

/// Command line argument Struct used by clap to parse CLI arguments.
#[derive(Parser, Serialize, Deserialize)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the image file.
    #[arg(required = true, index = 1)]
    pub image: PathBuf,
    /// Measure of centrality to be used to analyze an image.
    #[arg(short, long, default_value_t = Centrality::Prevalent)]
    pub centrality: Centrality,
    /// Output format for color themes.
    #[arg(short, long, default_value_t = OutputFormat::JSON)]
    pub serialization_format: OutputFormat,

    #[command(flatten)]
    pub color_themes: ColorThemeOption,
    /// Level of logging
    #[arg(short, long, default_value_t = 0)]
    pub log_level: usize,
}

#[derive(Args, Serialize, Deserialize)]
#[group(multiple = false)]
pub struct ColorThemeOption {
    /// Make color selected by the centrality darker.
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u8).range(0 ..= 100))]
    pub darker: u8,
    /// Make color selected by the centrality lighter.
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u8).range(0 ..= 100))]
    pub lighter: u8,
    /// Get the complementary color of the color selected by the centrality.
    #[arg(long, default_value_t = false)]
    pub complementary: bool,
    /// Get the highest contrasting color of the color selected by the centrality.
    #[arg(long, default_value_t = false)]
    pub contrast: bool,
    /// Change the angle of the color selected by the centrality
    #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u16).range(0 ..= 360))]
    pub hue_offset: u16,
    /// Get color scheme comprised of three equally spaced colors around the color wheel based on the centrality.
    #[arg(long, default_value_t = false)]
    pub triadic: bool,
    /// Get color scheme comprised of four equally spaced colors around the color wheel based on the centrality.
    #[arg(long, default_value_t = false)]
    pub quadratic: bool,
    /// Get color scheme comprised of two colors selected by the centrality and their complementary values.
    #[arg(long, default_value_t = false)]
    pub tetratic: bool,
    /// Get the two colors that sit next to the color selected by the centrality.
    #[arg(long, default_value_t = false)]
    pub analogous: bool,
    /// Get the two colors that sit next to the complement of the color selected by the centrality.
    #[arg(long, default_value_t = false)]
    pub split_complementary: bool,
    /// Number of colors with the same hue, but with a different saturation/lightness based on the color selected by the centrality.
    #[arg(long, default_value_t = 0)]
    pub monochromatic: u8,
    /// Number of colors, based on the color selected by the centrality, blended from the given color to black.
    #[arg(long, default_value_t = 0)]
    pub shades: u8,
    /// Number of colors, based on the color selected by the centrality, blended from the given color to white.
    #[arg(long, default_value_t = 0)]
    pub tints: u8,
    /// Number of colors, based on the color selected by the centrality, blended from the given color to gray.
    #[arg(long, default_value_t = 0)]
    pub tones: u8,
    /// Number of colors, based on two colors selected by the centrality, interpolated together.
    #[arg(long, default_value_t = 0)]
    pub blends: u8,
}

impl Display for ColorThemeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let darker = match self.darker {
            0 => "",
            _ => &format!("-Darker {}", self.darker),
        };
        let lighter = match self.lighter {
            0 => "",
            _ => &format!("-Lighter {}", self.lighter),
        };
        let complementary = match self.complementary {
            true => "-Complementary",
            false => "",
        };
        let contrast = match self.contrast {
            true => "-Contrast",
            false => "",
        };
        let hue_offset = match self.hue_offset {
            0 => "",
            _ => &format!("-HueOffset {}", self.hue_offset),
        };
        let triadic = match self.triadic {
            true => "-Triadic",
            false => "",
        };
        let quadratic = match self.quadratic {
            true => "-Quadratic",
            false => "",
        };
        let tetratic = match self.tetratic {
            true => "-Tetratic",
            false => "",
        };
        let analogous = match self.analogous {
            true => "-Analogous",
            false => "",
        };
        let split_complementary = match self.split_complementary {
            true => "-SplitComplementary",
            false => "",
        };
        let monochromatic = match self.monochromatic {
            0 => "",
            _ => &format!("-Monochromatic {}", self.monochromatic),
        };
        let shades = match self.shades {
            0 => "",
            _ => &format!("-Shades {}", self.shades),
        };
        let tints = match self.tints {
            0 => "",
            _ => &format!("-Tints {}", self.tints),
        };
        let tones = match self.tones {
            0 => "",
            _ => &format!("-Tones {}", self.tones),
        };
        let blends = match self.blends {
            0 => "",
            _ => &format!("-Blends {}", self.blends),
        };

        write!(f, "{darker}{lighter}{complementary}{contrast}{hue_offset}{triadic}{quadratic}{tetratic}{analogous}{split_complementary}{monochromatic}{shades}{tints}{tones}{blends}")
    }
}

/// Output format for [`color_scheme_generator::theme_calculation::ColorTheme`].
#[derive(Clone, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    JSON,
    YAML,
    TEXT,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::JSON => write!(f, "json"),
            OutputFormat::YAML => write!(f, "yaml"),
            OutputFormat::TEXT => write!(f, "text"),
        }
    }
}

/// Measures of centrality to generate ColorTheme.
#[derive(PartialEq, Copy, Clone, ValueEnum, Serialize, Deserialize)]
pub enum Centrality {
    /// Takes the sum of the pixels and divides by the amount of pixels in an image.
    Average,
    /// Sort the pixels and takes the middlemost pixel in an image.
    Median,
    /// Get the most repeating pixels in an image.
    Prevalent,
}

impl Display for Centrality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Centrality::Average => write!(f, "average"),
            Centrality::Median => write!(f, "median"),
            Centrality::Prevalent => write!(f, "prevalent"),
        }
    }
}

impl FromStr for Centrality {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "average" => Ok(Centrality::Average),
            "median" => Ok(Centrality::Median),
            "prevalent" => Ok(Centrality::Prevalent),
            _ => Err(Error.into()),
        }
    }
}

/// Struct representation for the [`image::Rgb<u8>`] type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RGB {
    /// Red RGB sub-pixel.
    pub red: u8,
    /// Green RGB sub-pixel.
    pub green: u8,
    /// Blue RGB sub-pixel.
    pub blue: u8,
}

impl Display for RGB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

impl FromStr for RGB {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = regex::Regex::new(r"^#[0123456789AaBbCcDdEeFf]{6}$").unwrap();
        match regex.is_match(s) {
            true => {
                let hex = s.as_bytes();
                let red = hex_to_rgb(hex[1], hex[2])?;
                let green = hex_to_rgb(hex[3], hex[4])?;
                let blue = hex_to_rgb(hex[5], hex[6])?;
                
                Ok(RGB { red, green, blue })
            }
            false => {
                Err(anyhow::anyhow!(
                    "Inputted string is not a valid hexadecimal RGB value. Example: #FFFFFF"
                ))
            }
        }
    }
}

fn hex_to_rgb(msd: u8, lsd: u8) -> anyhow::Result<u8> {

    let leading = (u16::from(char_to_u8(msd as char)?)) << 4;
    let smallest = u16::from(char_to_u8(lsd as char)?);
    println!("{:08b} {:08b}", leading, smallest);
    let ret = leading + smallest;
    Ok(u8::try_from(ret)?)

}

fn char_to_u8(c: char) -> anyhow::Result<u8> {

    let x =c.to_ascii_uppercase() as u8;

    match x.to_ascii_uppercase() as u8 {
        48..=58 => Ok(x   - 48),
        65..=90 => Ok(x - 55),
        _ => Err(anyhow::anyhow!("Character cannot be converted to u8."))
    }
}

/// Application Name used for XDG compliant directory structure.
pub const APP_NAME: &str = "color_scheme_generator";

/// Command line executable name for gamut-cli.
pub const GAMUT_CLI_NAME: &str = "gamut-cli";

pub struct Wallpaper {
    pub path: PathBuf,
    pub centrality: Centrality,
}
