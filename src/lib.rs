#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![deny(clippy::nursery)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use image::imageops::{FilterType, crop_imm, overlay, resize};
use image::{DynamicImage, GenericImageView, RgbaImage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TileShape {
    columns: u32,
    rows: u32,
}

impl TileShape {
    const SQUARE: Self = Self {
        columns: 1,
        rows: 1,
    };
    const LANDSCAPE: Self = Self {
        columns: 2,
        rows: 1,
    };
    const PORTRAIT: Self = Self {
        columns: 1,
        rows: 2,
    };
}

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

    let images = inputs
        .iter()
        .map(|path| {
            image::open(path)
                .with_context(|| format!("failed to open input image: {}", path.display()))
        })
        .collect::<Result<Vec<_>>>()?;
    let shapes: Vec<_> = images.iter().map(tile_shape).collect();
    let cell_count = shapes.iter().try_fold(0_u32, |total, shape| {
        total
            .checked_add(shape.columns * shape.rows)
            .context("too many input images")
    })?;
    let columns = options.columns.unwrap_or_else(|| auto_columns(cell_count));
    if columns == 0 {
        bail!("column count must be greater than zero");
    }
    if shapes.iter().any(|shape| shape.columns > columns) {
        bail!("column count must be at least 2 for landscape images");
    }

    let (placements, rows) = arrange_tiles(&shapes, columns)?;
    let width = columns
        .checked_mul(options.tile_size)
        .context("output image width is too large")?;
    let height = rows
        .checked_mul(options.tile_size)
        .context("output image height is too large")?;
    let mut grid = RgbaImage::new(width, height);

    for ((source, shape), (column, row)) in images.iter().zip(&shapes).zip(placements) {
        let tile = crop_tile(source, *shape, options.tile_size);
        let x = column * options.tile_size;
        let y = row * options.tile_size;
        overlay(&mut grid, &tile, i64::from(x), i64::from(y));
    }

    DynamicImage::ImageRgba8(grid)
        .save(output)
        .with_context(|| format!("failed to save output image: {}", output.display()))
}

fn auto_columns(count: u32) -> u32 {
    (f64::from(count).sqrt().ceil() as u32).max(1)
}

fn tile_shape(image: &DynamicImage) -> TileShape {
    let (width, height) = image.dimensions();
    if u64::from(width) * 10 >= u64::from(height) * 17 {
        TileShape::LANDSCAPE
    } else if u64::from(height) * 10 >= u64::from(width) * 17 {
        TileShape::PORTRAIT
    } else {
        TileShape::SQUARE
    }
}

fn arrange_tiles(shapes: &[TileShape], columns: u32) -> Result<(Vec<(u32, u32)>, u32)> {
    let columns_usize = usize::try_from(columns).context("column count is too large")?;
    let mut occupied: Vec<Vec<bool>> = Vec::new();
    let mut placements = Vec::with_capacity(shapes.len());

    for shape in shapes {
        let mut row = 0_usize;
        let shape_columns = usize::try_from(shape.columns).expect("tile shape is small");
        let shape_rows = usize::try_from(shape.rows).expect("tile shape is small");
        'search: loop {
            while occupied.len() < row + shape_rows {
                occupied.push(vec![false; columns_usize]);
            }
            for column in 0..=columns_usize - shape_columns {
                let fits = (row..row + shape_rows)
                    .all(|y| (column..column + shape_columns).all(|x| !occupied[y][x]));
                if fits {
                    for occupied_row in occupied.iter_mut().skip(row).take(shape_rows) {
                        occupied_row[column..column + shape_columns].fill(true);
                    }
                    placements.push((
                        u32::try_from(column).context("column index is too large")?,
                        u32::try_from(row).context("row index is too large")?,
                    ));
                    break 'search;
                }
            }
            row += 1;
        }
    }

    let rows = u32::try_from(occupied.len()).context("output image height is too large")?;
    Ok((placements, rows))
}

fn crop_tile(image: &DynamicImage, shape: TileShape, size: u32) -> RgbaImage {
    let (width, height) = image.dimensions();
    let target_width = shape.columns;
    let target_height = shape.rows;
    let (crop_width, crop_height) = if u64::from(width) * u64::from(target_height)
        > u64::from(height) * u64::from(target_width)
    {
        (height * target_width / target_height, height)
    } else {
        (width, width * target_height / target_width)
    };
    let x = (width - crop_width) / 2;
    let y = (height - crop_height) / 2;
    let rgba = image.to_rgba8();
    let cropped = crop_imm(&rgba, x, y, crop_width, crop_height).to_image();
    resize(
        &cropped,
        size * shape.columns,
        size * shape.rows,
        FilterType::Lanczos3,
    )
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
                save_solid(&path, 8, 5, color);
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
            let color = if (1..5).contains(&x) {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            };
            for y in 0..2 {
                image.put_pixel(x, y, color);
            }
        }

        let image = DynamicImage::ImageRgba8(image);
        let tile = crop_tile(&image, tile_shape(&image), 2);
        assert_eq!(tile.dimensions(), (4, 2));
        assert!(tile.pixels().all(|pixel| *pixel == Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn wide_and_tall_images_occupy_two_cells() {
        let dir = tempdir().unwrap();
        let wide = dir.path().join("wide.png");
        let square = dir.path().join("square.png");
        let tall = dir.path().join("tall.png");
        save_solid(&wide, 8, 3, Rgba([255, 0, 0, 255]));
        save_solid(&square, 4, 4, Rgba([0, 255, 0, 255]));
        save_solid(&tall, 3, 8, Rgba([0, 0, 255, 255]));
        let output = dir.path().join("grid.png");

        create_grid(
            &[wide, square, tall],
            &output,
            GridOptions {
                columns: Some(3),
                tile_size: 4,
            },
        )
        .unwrap();

        let result = image::open(output).unwrap().to_rgba8();
        assert_eq!(result.dimensions(), (12, 12));
        assert_eq!(*result.get_pixel(1, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*result.get_pixel(5, 1), Rgba([255, 0, 0, 255]));
        assert_eq!(*result.get_pixel(9, 1), Rgba([0, 255, 0, 255]));
        assert_eq!(*result.get_pixel(1, 5), Rgba([0, 0, 255, 255]));
        assert_eq!(*result.get_pixel(1, 9), Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn ratio_of_exactly_one_point_seven_is_rectangular() {
        let landscape = DynamicImage::ImageRgba8(RgbaImage::new(17, 10));
        let portrait = DynamicImage::ImageRgba8(RgbaImage::new(10, 17));
        assert_eq!(tile_shape(&landscape), TileShape::LANDSCAPE);
        assert_eq!(tile_shape(&portrait), TileShape::PORTRAIT);
    }

    #[test]
    fn ratio_below_one_point_seven_is_square() {
        let landscape = DynamicImage::ImageRgba8(RgbaImage::new(169, 100));
        let portrait = DynamicImage::ImageRgba8(RgbaImage::new(100, 169));
        assert_eq!(tile_shape(&landscape), TileShape::SQUARE);
        assert_eq!(tile_shape(&portrait), TileShape::SQUARE);
    }

    #[test]
    fn chooses_near_square_column_count() {
        assert_eq!(auto_columns(1), 1);
        assert_eq!(auto_columns(2), 2);
        assert_eq!(auto_columns(4), 2);
        assert_eq!(auto_columns(5), 3);
    }
}
