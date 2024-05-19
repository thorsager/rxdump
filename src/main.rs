use clap::Parser;
use std::fs::File;
use std::io;
use std::io::prelude::*;

const LINE_BYTES: usize = 16;

#[derive(Parser)]
#[command(version,about,long_about = None)]
struct Cli {
    filename: String,
    #[arg(short, long, value_name = "BYTES")]
    word_size: Option<usize>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let mut f = match File::open(&cli.filename) {
        Err(e) => panic!("could not open {}: {}", cli.filename, e),
        Ok(f) => f,
    };

    let word_size: usize = cli.word_size.unwrap_or(2);
    let line_words: usize = LINE_BYTES / word_size;
    let hex_length: usize = word_size * 2 * line_words + line_words;

    let mut buffer = [0; LINE_BYTES];
    let mut offset = 0;
    let mut last_was_all_zero = false;
    let mut skipped_lines = 0;
    loop {
        let n = match f.read(&mut buffer) {
            Ok(size) => size,
            Err(e) => {
                println!("while reading bufer: {}", e);
                0
            }
        };
        if n == 0 {
            break;
        }
        offset += n;
        let is_all_zero = all_zero(&buffer);

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
    }

    Ok(())
}

fn all_zero(line: &[u8]) -> bool {
    line.iter().all(|&x| x == 0)
}

fn word_as_hex(word: &[u8]) -> String {
    let mut wds: String = String::new();
    for (_, byte) in word.iter().enumerate() {
        let letter = format!("{:02x}", byte);
        wds += &letter;
    }
    wds
}

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
