use clap::Parser;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

const LINE_BYTES: usize = 16;

#[derive(Parser)]
#[command(version,about,long_about = None)]
struct Cli {
    /// Input filename
    filename: String,

    /// Do not skip all zero lines
    #[arg(long, action)]
    show_zero_lines: bool,

    /// Number of bytes in a "word"
    #[arg(short, long, value_name = "BYTES")]
    word_size: Option<usize>,

    /// Offset from which to start reading file (hexadecimal value prefix with '0x')
    #[arg(short, long, value_name = "BYTES")]
    offset: Option<String>,

    /// Limit of bytes to read from file (hexadecimal value prefix with '0x')
    #[arg(short, long, value_name = "BYTES")]
    limit: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let mut f = match File::open(&cli.filename) {
        Err(e) => {
            println!("could not open {}: {}", cli.filename, e);
            std::process::exit(2);
        }
        Ok(f) => f,
    };

    let word_size: usize = cli.word_size.unwrap_or(2);
    let line_words: usize = LINE_BYTES / word_size;
    let hex_length: usize = word_size * 2 * line_words + line_words;
    let skip_lines: bool = !cli.show_zero_lines;

    let mut buffer = [0; LINE_BYTES];
    let mut offset: usize = 0;
    let mut limit: usize = 0;
    let mut last_was_all_zero = false;
    let mut skipped_lines = 0;

    // possition to offset if passed
    if cli.offset.is_some() {
        let offset_str = cli.offset.unwrap();
        let pos = match as_u64(&offset_str) {
            Err(e) => {
                println!("invalid offset value '{}': {}", &offset_str, e);
                std::process::exit(3);
            }
            Ok(v) => v,
        };
        match f.seek(SeekFrom::Start(pos)) {
            Err(e) => {
                println!(
                    "could not seek to pos {} on file {}: {}",
                    pos, cli.filename, e
                );
                std::process::exit(3);
            }
            Ok(n) => offset += usize::try_from(n).unwrap(),
        }
        println!("**")
    };

    // calculate limit if passed
    if cli.limit.is_some() {
        let limit_str = cli.limit.unwrap();
        limit = match as_u64(&limit_str) {
            Err(e) => {
                println!("invalid limit value '{}': {}", &limit_str, e);
                std::process::exit(3);
            }
            Ok(v) => v.try_into().unwrap(),
        };
    }

    // read through file
    loop {
        let mut n = match f.read(&mut buffer) {
            Ok(size) => size,
            Err(e) => {
                println!("while reading bufer: {}", e);
                0
            }
        };
        if n == 0 {
            // we are EOF
            if skipped_lines > 0 {
                println!("*")
            }
            println!("{:08x}", (offset - n));
            break;
        }
        if limit != 0 && (offset + n) >= limit {
            n = limit - offset
        }

        offset += n;
        let mut is_all_zero = false;
        if skip_lines {
            is_all_zero = all_zero(&buffer);
        }

        if is_all_zero && last_was_all_zero && (n == buffer.len()) {
            skipped_lines += 1;
            continue;
        }

        let mut hex: String = String::new();
        let mut ascii: String = String::new();
        for (i, word) in buffer[0..n].chunks(word_size).enumerate() {
            hex += &word_as_hex(word);
            if i < n {
                hex += " "
            }
            ascii += &word_as_ascii(word);
        }
        if skipped_lines > 0 {
            skipped_lines = 0;
            println!("*")
        }

        println!("{:08x}  {: <3$} |{}|", (offset - n), hex, ascii, hex_length);
        last_was_all_zero = is_all_zero;

        if offset == limit {
            println!("**");
            break;
        }
    }
}

fn as_u64(s: &String) -> Result<u64, std::num::ParseIntError> {
    if s.starts_with("0x") {
        let h = s.trim_start_matches("0x");
        u64::from_str_radix(h, 16)
    } else {
        u64::from_str_radix(s.as_str(), 10)
    }
}

// all_zero will return true if all bytes in a byte array is zero
fn all_zero(line: &[u8]) -> bool {
    line.iter().position(|&x| x != 0) == None
}

// word_as_hex converts an array of bytes to a hex string, it will pad
// the hexvalue of each byte witn '0'
fn word_as_hex(word: &[u8]) -> String {
    let mut wds: String = String::new();
    for (_, byte) in word.iter().enumerate() {
        let letter = format!("{:02x}", byte);
        wds += &letter;
    }
    wds
}

// word_as_ascii convets an array of bytes to a printable ascii string
// replacing non-printable chars with '.'
fn word_as_ascii(word: &[u8]) -> String {
    let mut a: String = String::new();
    for (_, b) in word.iter().enumerate() {
        if *b >= 0x20 && *b < 0x7f {
            a.push(*b as char)
        } else {
            a.push('.')
        }
    }
    a
}
