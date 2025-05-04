use std::{borrow::Cow, fs::File, path::Path};

use clap::Parser;
use gif::{ColorOutput, DecodeOptions, Encoder, Frame, Repeat};

#[derive(Parser)]
#[command(name = "Pointillist")]
#[command(version = "0.1")]
#[command(about = "Turns any gif into a pointillist style gif.", long_about = None)]
pub struct Args {
    /// Path to the input GIF file
    #[arg(short, long)]
    pub in_path: String,

    /// Path to the output GIF file
    #[arg(short, long)]
    pub out_path: String,

    /// Size of the blocks to cluster pixels into
    #[arg(short, long, default_value_t = 8)]
    pub block_size: usize,

    /// How much padding to add between the circles
    #[arg(short, long, default_value_t = 2)]
    pub padding: u32,

    /// Maximum radius of the circles
    #[arg(short, long, default_value_t = 8)]
    pub radius: u32,

    /// Delay of the frames in the output GIF
    #[arg(short, long, default_value_t = 5)]
    pub delay: u16,
}

#[derive(Debug)]
pub struct DotFrame {
    pub width: u16,
    pub height: u16,

    // Some "key" value, can be arbitrary for now e.g. brightness, hue
    pub buffer: Vec<usize>,
}

pub struct GifFrame {
    pub width: u16,
    pub height: u16,
    /// The pixel data of the GIF frame in RGBA format.
    pub buffer: Vec<(u8, u8, u8, u8)>,
}

fn extract_gif_frames<P: AsRef<Path>>(path: P) -> Result<Vec<GifFrame>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(ColorOutput::RGBA);
    let mut decoder = decoder
        .read_info(file)
        .map_err(|e| format!("Failed to read GIF info: {}", e))?;

    let mut frames = Vec::new();

    while let Some(frame) = decoder
        .read_next_frame()
        .map_err(|e| format!("Failed to read frame: {}", e))?
    {
        let width = frame.width;
        let height = frame.height;

        debug_assert!(
            frame.buffer.len() == width as usize * height as usize * 4,
            "Buffer length mismatch"
        );

        let mut buffer = Vec::with_capacity(width as usize * height as usize * 4);

        let mut i = 0;
        while i + 3 < frame.buffer.len() {
            let rgba = (
                frame.buffer[i],
                frame.buffer[i + 1],
                frame.buffer[i + 2],
                frame.buffer[i + 3],
            );
            buffer.push(rgba);
            i += 4;
        }

        frames.push(GifFrame {
            width,
            height,
            buffer,
        });
    }

    Ok(frames)
}

fn convert_to_dots(
    frames: Vec<GifFrame>,
    block_size: usize,
    key_func: impl Fn(&(u8, u8, u8, u8)) -> usize,
) -> Vec<DotFrame> {
    let mut dot_frames = Vec::new();
    for frame in frames {
        // For every frame we want to cluster the pixels into blocks of size block_size x block_size
        // and calculate the average brightness of each block.
        let width = frame.width as usize;
        let mut blocks = Vec::new();

        for y in (0..frame.height as usize).step_by(block_size) {
            for x in (0..frame.width as usize).step_by(block_size) {
                let mut total = 0;
                let mut count = 0;

                for dy in 0..block_size {
                    for dx in 0..block_size {
                        let px = x + dx;
                        let py = y + dy;
                        if px >= frame.width as usize || py >= frame.height as usize {
                            continue;
                        }

                        let index = py * width + px;
                        if index >= frame.buffer.len() {
                            continue;
                        }

                        let pixel = frame.buffer[index];
                        total += key_func(&pixel);
                        count += 1;
                    }
                }

                let avg = if count > 0 { total / count } else { 0 };
                blocks.push(avg);
            }
        }

        // Now we can create a new DotFrame with the blocks
        // and the width and height of the frame
        let blocks_w = (frame.width as usize + block_size - 1) / block_size;
        let blocks_h = (frame.height as usize + block_size - 1) / block_size;
        let expected_len = blocks_w * blocks_h;

        debug_assert!(
            blocks.len() == expected_len,
            "Expected: {}, but got: {}",
            expected_len,
            blocks.len()
        );

        dot_frames.push(DotFrame {
            width: blocks_w as u16,
            height: blocks_h as u16,
            buffer: blocks,
        });
    }

    dot_frames
}

pub fn write_circles_gif(
    path: &str,
    frames: &[DotFrame],
    padding: u32,
    max_radius: u32,
    max_value: usize,
    delay: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(!frames.is_empty(), "Need at least one frame");

    let grid_w = frames[0].width as u32;
    let grid_h = frames[0].height as u32;
    let img_w = grid_w * (2 * max_radius + padding) + padding;
    let img_h = grid_h * (2 * max_radius + padding) + padding;

    let mut image = File::create(path)?;

    // 2) Two-color global palette: black, then white
    let palette: &[u8] = &[
        0, 0, 0, // index 0 == black
        255, 255, 255, // index 1 == white
    ];

    let mut encoder = Encoder::new(&mut image, img_w as u16, img_h as u16, palette)?;
    encoder.set_repeat(Repeat::Infinite)?;

    let frame_buf_size = (img_w * img_h) as usize;
    let mut pixels = vec![0u8; frame_buf_size];

    for df in frames {
        pixels.fill(2); // Fill with index 2 for transparent pixels

        for row in 0..grid_h {
            for col in 0..grid_w {
                let idx = (row * grid_w + col) as usize;
                let val = df.buffer[idx];
                let r = (val as f32 / max_value.max(1) as f32) * (max_radius as f32);
                let r2 = r * r;

                let cx = padding as f32
                    + (col as f32 * (2.0 * max_radius as f32 + padding as f32))
                    + max_radius as f32;
                let cy = padding as f32
                    + (row as f32 * (2.0 * max_radius as f32 + padding as f32))
                    + max_radius as f32;

                let x0 = ((cx - r).max(0.0).floor()) as u32;
                let x1 = ((cx + r).min((img_w - 1) as f32).ceil()) as u32;
                let y0 = ((cy - r).max(0.0).floor()) as u32;
                let y1 = ((cy + r).min((img_h - 1) as f32).ceil()) as u32;

                for y in y0..=y1 {
                    for x in x0..=x1 {
                        let dx = x as f32 - cx;
                        let dy = y as f32 - cy;
                        if dx * dx + dy * dy <= r2 {
                            let pix_idx = (y * img_w + x) as usize;
                            pixels[pix_idx] = 1;
                        }
                    }
                }
            }
        }

        let frame = Frame {
            width: img_w as u16,
            height: img_h as u16,
            buffer: Cow::Borrowed(&pixels),
            delay,
            transparent: Some(2), // index 2 == for transparent pixels
            dispose: gif::DisposalMethod::Background,
            ..Frame::default()
        };

        encoder.write_frame(&frame)?;
    }

    Ok(())
}

#[inline(always)]
pub fn human_perceived_brightness(r: u8, g: u8, b: u8) -> u8 {
    (0.299 * (r as f32).powi(2) + 0.587 * (g as f32).powi(2) + 0.114 * (b as f32).powi(2))
        .sqrt()
        .round() as u8
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let frames = extract_gif_frames(&args.in_path)?;
    let dot_frames = convert_to_dots(frames, args.block_size, |(r, g, b, a)| {
        if *a < 128 {
            0 // Make fully transparent pixels have zero brightness
        } else {
            // Scale brightness by alpha
            (human_perceived_brightness(*r, *g, *b) as f32 * (*a as f32 / 255.0)) as usize
        }
    });

    let max_value = dot_frames
        .iter()
        .flat_map(|f| f.buffer.iter())
        .cloned()
        .max()
        .unwrap_or(1);

    write_circles_gif(
        &args.out_path,
        &dot_frames,
        args.padding,
        args.radius,
        max_value,
        args.delay,
    )?;

    Ok(())
}
