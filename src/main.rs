use std::{
    char,
    fs::{read_to_string, File},
    io::Read,
    process::exit,
    str::FromStr,
};

use ab_glyph::{Font, FontVec, ScaleFont};
use anyhow::Result;
use clap::{command, Parser};
use image::{DynamicImage, GenericImageView, ImageReader, Rgb, RgbImage};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_text_mut},
    rect::Rect,
};
use indicatif::ProgressBar;
use rand::seq::SliceRandom;

fn main() {
    let args = Args::parse();

    // validate options
    if !args.textfile.is_empty() && !args.character.is_empty() {
        println!("You cannot have both flags at the same time: --character, --textfile");
        exit(0);
    }

    let charset =
        Charset::from_str(&args.charset).expect("charset is validated during argument parsing");
    let mut characters = get_characters(charset);

    let background_color = get_rgb_from_hex(&args.background).unwrap_or_else(|_| {
        println!("Invalid background color: {}", args.background);
        exit(0);
    });

    if !args.custom_charset.is_empty() {
        characters = read_to_string(args.custom_charset)
            .unwrap_or_else(|e| {
                println!("Unable to read custom charset file: {}", e);
                exit(0);
            })
            .trim()
            .chars()
            .collect();
    }

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

    // load font
    let font = get_font(&args.font).unwrap_or_else(|e| {
        println!("Unable to read font file: {}", e);
        exit(0)
    });

    // load text and initialize character iterator
    let text = match args.textfile.as_str() {
        "" => String::new(),
        filename => sanatize_text(read_to_string(filename).unwrap_or_else(|e| {
            println!("Could not read text file: {}", e);
            exit(0);
        })),
    };
    let mut text_chars = text.chars().cycle();

    let image_width = input_image.width();
    let image_height = input_image.height();

    let mut output_image = RgbImage::new(image_width, image_height);

    draw_filled_rect_mut(
        &mut output_image,
        Rect::at(0, 0).of_size(image_width, image_height),
        background_color,
    );

    let mut rng = rand::thread_rng();

    let scaled_font = font.as_scaled(args.font_size);
    let glyph_height = scaled_font.height() - scaled_font.line_gap();

    let total_lines = input_image.height() / glyph_height.ceil() as u32;

    let progress_bar = ProgressBar::new(total_lines as u64 + 1);

    let mut y = 0;
    while y < input_image.height() {
        let mut x = 0;
        while x < input_image.width() {
            let glyph = match text_chars.next() {
                None => match args.character.is_empty() {
                    // use random character
                    true => *characters
                        .choose(&mut rng)
                        .expect("vec should never be empty"),
                    false => args.character.chars().next().unwrap(),
                },
                Some(c) => c,
            };

            let glyph_id = font.glyph_id(glyph);
            let glyph_width =
                scaled_font.h_advance(glyph_id) + scaled_font.h_side_bearing(glyph_id);
            let image_section = input_image.crop_imm(x, y, glyph_width as u32, glyph_height as u32);
            let color = get_average_color(image_section);

            draw_text_mut(
                &mut output_image,
                color,
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                args.font_size,
                &font,
                &glyph.to_string(),
            );
            x += glyph_width as u32;
        }
        y += glyph_height as u32;
        progress_bar.inc(1);
    }

    if output_image.save(&args.outfile).is_err() {
        println!("Couldn't write to file: {}", args.outfile);
    }
    progress_bar.finish();
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
    #[arg(short, long, default_value_t = String::new())]
    character: String,
    #[arg(long, default_value_t = String::new())]
    textfile: String,
    #[arg(long, default_value_t = String::from("latin"), ignore_case = true, value_parser = [
        "latin",
        "cyrillic",
        "runic",
        "hebrew",
        "hiragana",
        "katakana",
        "hangul",
        "cjkunified",
        "greek",
        "emoticons",
        "decimal",
        "hexadecimal",
        "binary",
        "braille",
        "playingcards",
    ])]
    charset: String,
    #[arg(long, default_value_t = String::new())]
    custom_charset: String,
    #[arg(short, long, default_value_t = String::from("#000000"))]
    background: String,
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

fn get_rgb_from_hex(hex: &str) -> Result<Rgb<u8>> {
    let hex = hex.replace("#", "");

    if hex.len() < 6 {
        anyhow::bail!("invalid hex string");
    }

    let (r, rest) = hex.split_at(2);
    let (g, b) = rest.split_at(2);

    let r = u8::from_str_radix(r, 16)?;
    let g = u8::from_str_radix(g, 16)?;
    let b = u8::from_str_radix(b, 16)?;

    Ok(Rgb::from([r, g, b]))
}

fn sanatize_text(text: String) -> String {
    text.replace(|c: char| !c.is_alphabetic(), "")
}

enum Charset {
    Latin,
    Cyrillic,
    Runic,
    Hebrew,
    Hiragana,
    Katakana,
    Hangul,
    CkjUnified,
    Greek,
    Emoticons,
    Decimal,
    Binary,
    Hexadecimal,
    Braille,
    PlayingCards,
}

impl FromStr for Charset {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "latin" => Ok(Self::Latin),
            "cyrillic" => Ok(Self::Cyrillic),
            "runic" => Ok(Self::Runic),
            "hebrew" => Ok(Self::Hebrew),
            "hiragana" => Ok(Self::Hiragana),
            "katakana" => Ok(Self::Katakana),
            "hangul" => Ok(Self::Hangul),
            "cjkunified" => Ok(Self::CkjUnified),
            "greek" => Ok(Self::Greek),
            "emoticons" => Ok(Self::Emoticons),
            "decimal" => Ok(Self::Decimal),
            "hexadecimal" => Ok(Self::Hexadecimal),
            "binary" => Ok(Self::Binary),
            "braille" => Ok(Self::Braille),
            "playingcards" => Ok(Self::PlayingCards),
            _ => Err(()),
        }
    }
}

fn get_characters(charset: Charset) -> Vec<char> {
    match charset {
        Charset::Latin => ('\u{0041}'..='\u{007A}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Cyrillic => ('\u{0400}'..='\u{04FF}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Runic => ('\u{16A0}'..='\u{16FF}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Hebrew => ('\u{0590}'..='\u{05FF}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Hiragana => ('\u{3040}'..='\u{309F}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Hangul => ('\u{1100}'..='\u{11FF}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Katakana => ('\u{30A0}'..='\u{30FF}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::CkjUnified => ('\u{4E00}'..='\u{9FFF}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::Emoticons => ('\u{1F600}'..='\u{1F64F}').collect(),
        Charset::Decimal => ('0'..='9').collect(),
        Charset::Hexadecimal => {
            let mut hexadecimal: Vec<char> = ('0'..='9').collect();
            hexadecimal.extend('A'..='F');
            hexadecimal
        }
        Charset::Binary => vec!['0', '1'],
        Charset::Braille => ('\u{2800}'..='\u{28FF}').collect(),
        Charset::Greek => ('\u{0370}'..='\u{03E1}')
            .filter(|c| c.is_alphabetic())
            .collect(),
        Charset::PlayingCards => ('\u{1F0A0}'..='\u{1F0DF}')
            .filter(|c| {
                *c != '\u{1F0AF}' && *c != '\u{1F0B0}' && *c != '\u{1F0C0}' && *c != '\u{1F0D0}'
            })
            .collect(),
    }
}
