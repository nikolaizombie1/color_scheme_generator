#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use clap::ValueEnum;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

#[derive(PartialEq, Copy, Clone, ValueEnum)]
pub enum Centrality {
    Average,
    Median,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RGB {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorTheme {
    pub bar_color: RGB,
    pub workspace_color: RGB,
    pub text_color: RGB,
}

pub fn generate_color_theme(
    path: &PathBuf,
    centrality: Centrality,
    number_of_themes: u8,
) -> Vec<ColorTheme> {
    let pixels = image::io::Reader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .to_rgb8()
        .pixels()
        .copied()
        .collect::<Vec<_>>();
    let bar_color = match centrality {
        Centrality::Average => vec![average_pixel(&pixels)],
        Centrality::Median => vec![median_pixel(&pixels)],
        Centrality::Prevalent => prevalent_pixel(&pixels, number_of_themes),
    };
    match centrality {
        Centrality::Average | Centrality::Median => vec![(calculate_color_theme(&bar_color[0]))],
        Centrality::Prevalent => bar_color
            .par_iter()
            .map(calculate_color_theme)
            .collect::<Vec<_>>(),
    }
}

fn calculate_color_theme(rgb: &RGB) -> ColorTheme {
    let workspace_color = complementary_color(rgb);
    let text_color =
        if workspace_color.red > 128 && workspace_color.blue > 128 && workspace_color.green > 128 {
            RGB {
                red: 0,
                green: 0,
                blue: 0,
            }
        } else {
            RGB {
                red: 255,
                green: 255,
                blue: 255,
            }
        };
    ColorTheme {
        bar_color: rgb.clone(),
        workspace_color,
        text_color,
    }
}

fn average_pixel(pixels: &[image::Rgb<u8>]) -> RGB {
    RGB {
        red: u8::try_from(
            pixels
                .par_iter()
                .map(|p| usize::from(p.0[0]))
                .sum::<usize>()
                / pixels.len(),
        )
        .unwrap(),
        green: u8::try_from(
            pixels
                .par_iter()
                .map(|p| usize::from(p.0[1]))
                .sum::<usize>()
                / pixels.len(),
        )
        .unwrap(),
        blue: u8::try_from(
            pixels
                .par_iter()
                .map(|p| usize::from(p.0[2]))
                .sum::<usize>()
                / pixels.len(),
        )
        .unwrap(),
    }
}

fn median_pixel(pixels: &[image::Rgb<u8>]) -> RGB {
    RGB {
        red: median(&pixels.par_iter().map(|p| p.0[0]).collect::<Vec<_>>()),
        green: median(&pixels.par_iter().map(|p| p.0[1]).collect::<Vec<_>>()),
        blue: median(&pixels.par_iter().map(|p| p.0[2]).collect::<Vec<_>>()),
    }
}

fn median(color_slice: &[u8]) -> u8 {
    if color_slice.len() % 2 == 0 {
        let left_middle =
            color_slice[(((color_slice.len() as f64) / (2.0)) - 1.0).floor() as usize];
        let right_middle = color_slice[((color_slice.len() as f64) / (2.0)).floor() as usize];
        (((right_middle as f64) + (left_middle as f64)) / 2.0) as u8
    } else {
        color_slice[(((color_slice.len() as f64) / 2.0) - 1.0) as usize]
    }
}

fn prevalent_pixel(pixels: &[image::Rgb<u8>], number_of_themes: u8) -> Vec<RGB> {
    let mut pixel_prevalence_count = std::collections::HashMap::new();
    for pixel in pixels.iter() {
        let count = pixel_prevalence_count.entry(pixel).or_insert(0);
        *count += 1;
    }
    let most_prevalent = pixel_prevalence_count
        .par_iter()
        .map(|x| (x.0, x.1))
        .collect::<Vec<_>>();
    if most_prevalent.len() > number_of_themes as usize {
        most_prevalent[0..(number_of_themes as usize)]
            .par_iter()
            .map(|x| RGB {
                red: x.0[0],
                green: x.0[1],
                blue: x.0[2],
            })
            .collect::<Vec<_>>()
    } else {
        most_prevalent
            .par_iter()
            .map(|x| RGB {
                red: x.0[0],
                green: x.0[1],
                blue: x.0[2],
            })
            .collect::<Vec<_>>()
    }
}

fn complementary_color(rgb: &RGB) -> RGB {
    RGB {
        red: 255 - rgb.red,
        green: 255 - rgb.green,
        blue: 255 - rgb.blue,
    }
}
