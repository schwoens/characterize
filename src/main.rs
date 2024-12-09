use std::{fs::File, io::Read, process::exit};

use ab_glyph::{Font, FontVec, ScaleFont};
use anyhow::Result;
use clap::{command, Parser};
use image::{DynamicImage, GenericImageView, ImageReader, Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;

fn main() {
    let args = Args::parse();

    let input_image = match ImageReader::open(&args.filename) {
        Ok(file) => file.decode().expect("Failed to decode image"),
        Err(_) => {
            println!("No such file: {}", &args.filename);
            return;
        }
    };

    let font = get_font(&args.font).unwrap_or_else(|_| {
        println!("Unable to read font file: {}", &args.font);
        exit(0)
    });

    let font_scale = font.as_scaled(args.font_size).scale().round();

    let mut output_image = RgbImage::new(input_image.width(), input_image.height());

    let mut y = 0;
    while y < input_image.height() {
        let mut x = 0;
        while x < input_image.width() {
            let image_section =
                input_image.crop_imm(x, y, font_scale.x as u32, font_scale.y as u32);

            let color = get_average_color(image_section);
            draw_text_mut(
                &mut output_image,
                color,
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                font_scale,
                &font,
                "#",
            );
            x += font_scale.x as u32;
        }
        y += font_scale.y as u32;
    }

    if output_image.save(&args.outfile).is_err() {
        println!("Couldn't write to file: {}", args.outfile);
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    filename: String,
    outfile: String,
    #[arg(short, long)]
    font: String,
    #[arg(long, default_value_t = 12.0)]
    font_size: f32,
}

fn get_font(filename: &str) -> Result<FontVec> {
    let mut file = File::open(filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(FontVec::try_from_vec(data)?)
}

fn get_average_color(image_section: DynamicImage) -> Rgb<u8> {
    let mut r: usize = 0;
    let mut g: usize = 0;
    let mut b: usize = 0;

    for pixel in image_section.pixels() {
        let color = pixel.2;

        r += color[0] as usize;
        g += color[1] as usize;
        b += color[2] as usize;
    }

    let pixel_amount = image_section.pixels().count();

    r /= pixel_amount;
    g /= pixel_amount;
    b /= pixel_amount;

    Rgb::from([r as u8, g as u8, b as u8])
}
