#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::nursery)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use grid_images::{GridOptions, create_grid};

/// Crop images to squares and arrange them in a grid.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Input image files, in display order
    #[arg(required = true, value_name = "IMAGE")]
    inputs: Vec<PathBuf>,

    /// Output image path (format is inferred from the extension)
    #[arg(short, long, default_value = "grid.png", value_name = "FILE")]
    output: PathBuf,

    /// Number of columns (defaults to a near-square layout)
    #[arg(short, long, value_name = "COUNT", value_parser = clap::value_parser!(u32).range(1..))]
    columns: Option<u32>,

    /// Width and height of each square tile in pixels
    #[arg(short = 's', long, default_value_t = 512, value_name = "PIXELS", value_parser = clap::value_parser!(u32).range(1..))]
    tile_size: u32,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let options = GridOptions {
        columns: cli.columns,
        tile_size: cli.tile_size,
    };

    create_grid(&cli.inputs, &cli.output, options)
}
