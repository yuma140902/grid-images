#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::nursery)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use image::imageops::{FilterType, crop_imm, overlay, resize};
use image::{DynamicImage, GenericImageView, RgbaImage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptions {
    pub columns: Option<u32>,
    pub tile_size: u32,
}

impl Default for GridOptions {
    fn default() -> Self {
        Self {
            columns: None,
            tile_size: 512,
        }
    }
}

/// Creates a grid image from `inputs` and saves it to `output`.
pub fn create_grid(inputs: &[PathBuf], output: &Path, options: GridOptions) -> Result<()> {
    if inputs.is_empty() {
        bail!("at least one input image is required");
    }
    if options.tile_size == 0 {
        bail!("tile size must be greater than zero");
    }

    let count = u32::try_from(inputs.len()).context("too many input images")?;
    let columns = options.columns.unwrap_or_else(|| auto_columns(count));
    if columns == 0 {
        bail!("column count must be greater than zero");
    }
    let rows = count.div_ceil(columns);
    let width = columns
        .checked_mul(options.tile_size)
        .context("output image width is too large")?;
    let height = rows
        .checked_mul(options.tile_size)
        .context("output image height is too large")?;
    let mut grid = RgbaImage::new(width, height);

    for (index, path) in inputs.iter().enumerate() {
        let source = image::open(path)
            .with_context(|| format!("failed to open input image: {}", path.display()))?;
        let tile = square_tile(&source, options.tile_size);
        let index = u32::try_from(index).expect("input count was checked above");
        let x = (index % columns) * options.tile_size;
        let y = (index / columns) * options.tile_size;
        overlay(&mut grid, &tile, i64::from(x), i64::from(y));
    }

    DynamicImage::ImageRgba8(grid)
        .save(output)
        .with_context(|| format!("failed to save output image: {}", output.display()))
}

fn auto_columns(count: u32) -> u32 {
    (f64::from(count).sqrt().ceil() as u32).max(1)
}

fn square_tile(image: &DynamicImage, size: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let side = width.min(height);
    let x = (width - side) / 2;
    let y = (height - side) / 2;
    let rgba = image.to_rgba8();
    let square = crop_imm(&rgba, x, y, side, side).to_image();
    resize(&square, size, size, FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use tempfile::tempdir;

    fn save_solid(path: &Path, width: u32, height: u32, color: Rgba<u8>) {
        RgbaImage::from_pixel(width, height, color)
            .save(path)
            .unwrap();
    }

    #[test]
    fn creates_grid_in_input_order_with_transparent_unused_cells() {
        let dir = tempdir().unwrap();
        let colors = [
            Rgba([255, 0, 0, 255]),
            Rgba([0, 255, 0, 255]),
            Rgba([0, 0, 255, 255]),
        ];
        let inputs: Vec<_> = colors
            .iter()
            .enumerate()
            .map(|(index, &color)| {
                let path = dir.path().join(format!("{index}.png"));
                save_solid(&path, 8, 4, color);
                path
            })
            .collect();
        let output = dir.path().join("grid.png");

        create_grid(
            &inputs,
            &output,
            GridOptions {
                columns: Some(2),
                tile_size: 4,
            },
        )
        .unwrap();

        let result = image::open(output).unwrap().to_rgba8();
        assert_eq!(result.dimensions(), (8, 8));
        assert_eq!(*result.get_pixel(1, 1), colors[0]);
        assert_eq!(*result.get_pixel(5, 1), colors[1]);
        assert_eq!(*result.get_pixel(1, 5), colors[2]);
        assert_eq!(*result.get_pixel(5, 5), Rgba([0, 0, 0, 0]));
    }

    #[test]
    fn center_crops_landscape_images() {
        let mut image = RgbaImage::new(6, 2);
        for x in 0..6 {
            let color = if (2..4).contains(&x) {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            };
            for y in 0..2 {
                image.put_pixel(x, y, color);
            }
        }

        let tile = square_tile(&DynamicImage::ImageRgba8(image), 2);
        assert!(tile.pixels().all(|pixel| *pixel == Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn chooses_near_square_column_count() {
        assert_eq!(auto_columns(1), 1);
        assert_eq!(auto_columns(2), 2);
        assert_eq!(auto_columns(4), 2);
        assert_eq!(auto_columns(5), 3);
    }
}
