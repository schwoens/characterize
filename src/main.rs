use std::{
    fs::{read_to_string, File},
    io::Read,
    process::exit,
};

use ab_glyph::{Font, FontVec, ScaleFont};
use anyhow::Result;
use clap::{command, Parser};
use image::{DynamicImage, GenericImageView, ImageReader, Rgb, RgbImage};
use imageproc::drawing::draw_text_mut;
use rand::Rng;

fn main() {
    let args = Args::parse();

    // open the image and decode it
    let mut input_image = match ImageReader::open(&args.filename) {
        Ok(file) => file.decode().unwrap_or_else(|_| {
            println!("Unsupported image format");
            exit(0);
        }),
        Err(_) => {
            println!("No such file: {}", &args.filename);
            exit(0);
        }
    };

    // scale the image if a non-default scale is set
    if args.scale != 1.0 {
        input_image = input_image.resize(
            (input_image.width() as f32 * args.scale).round() as u32,
            (input_image.height() as f32 * args.scale).round() as u32,
            image::imageops::FilterType::Nearest,
        );
    }

    let font = get_font(&args.font).unwrap_or_else(|_| {
        println!("Unable to read font file: {}", &args.font);
        exit(0)
    });

    let text = match args.textfile.as_str() {
        "" => String::new(),
        filename => sanatize_text(read_to_string(filename).unwrap_or_else(|_| {
            println!("Could not read text file: {}", args.textfile);
            exit(0);
        })),
    };

    let mut chars = text.chars().cycle();

    // get glyph width and height
    let scaled_font = font.as_scaled(args.font_size);
    let font_scale = scaled_font.scale;
    let glyph_id = font.glyph_id('@');

    let glyph_width = scaled_font.h_advance(glyph_id) + scaled_font.h_side_bearing(glyph_id);
    let glyph_height = scaled_font.height();

    let mut output_image = RgbImage::new(input_image.width(), input_image.height());

    let mut rng = rand::thread_rng();
    let upper_letter_range = b'A'..=b'Z';
    let lower_letter_range = b'a'..=b'z';
    let mut letters: Vec<u8> = upper_letter_range.collect();
    letters.append(&mut lower_letter_range.collect());

    let mut y = 0;
    while y < input_image.height() {
        let mut x = 0;
        while x < input_image.width() {
            let image_section = input_image.crop_imm(x, y, glyph_width as u32, glyph_height as u32);
            let color = get_average_color(image_section);

            let glyph = match chars.next() {
                None => {
                    let letter_index = rng.gen_range(0..letters.len());
                    letters[letter_index] as char
                }
                Some(c) => c,
            };

            draw_text_mut(
                &mut output_image,
                color,
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                font_scale,
                &font,
                &glyph.to_string(),
            );
            x += glyph_width as u32;
        }
        y += glyph_height as u32;
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
    #[arg(short, long, default_value_t = 1.0)]
    scale: f32,
    #[arg(short, long, default_value_t = String::from(""))]
    character: String,
    #[arg(long, default_value_t = String::from(""))]
    textfile: String,
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

fn sanatize_text(text: String) -> String {
    text.replace(" ", "")
        .replace("\n", "")
        .replace(".", "")
        .replace(",", "")
        .replace("-", "")
        .replace("\"", "")
        .replace("\'", "")
}
