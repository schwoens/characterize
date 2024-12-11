# Characterize

A CLI tool to make ascii art out of your images!

## Usage

```$ characterize <FILENAME> <OUTFILE> --font <FONT> [OPTIONS]```

Example image:  

![earth](https://github.com/user-attachments/assets/b25b7f77-5952-4dd9-8238-55cfc196986b)

### Random characters (default)

```$ characterize earth.jpg earth_out.jpg -f font.otf```

![earth_out_random_characters](https://github.com/user-attachments/assets/9b7a607c-5423-4245-a1ce-4e638c04005d)

**Note**: You should use a monospaced font to avoid overlapping characters.

### Set character

```$ characterize earth.jpg earth_out.jpg -f font.otf --character "A"```

![earth_out_set_character](https://github.com/user-attachments/assets/35326272-af16-44a2-8dc0-90579b77083a)

### Using a text file

```$ characterize earth.jpg earth_out.jpg -f font.otf --textfile lorem.txt```

![earth_out_textfile](https://github.com/user-attachments/assets/6121c49b-0f78-46e4-ba6f-7ebfe3a598ed)

**Note**: All non-alphabetic characters will be filtered out to ensure a gapless image.

### Up-/downscale image

```$ characterize earth.jpg earth_out.jpg -f font.otf --scale 2.5```

![earth_out_scaled](https://github.com/user-attachments/assets/6ea21cb4-9085-4656-9a85-ac783915eeb8)

### Set font size

```$ characterize earth.jpg earth_out.jpg -f font.otf --font-size 25.5``

![earth_out](https://github.com/user-attachments/assets/b968bd3d-ded5-40c6-95b3-200d519d7221)

Use `characterize --help` for more information

## Installation

**Prerequisites**: [Rust](https://rust-lang.org)

```
$ git clone https://github.com/schwoens/characterize && cd characterize
$ cargo build --release
```
Run executable:
```$ characterize/target/release/characterize```


