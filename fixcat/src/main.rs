use colored::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use fixcat::*;

fn main() -> std::io::Result<()> {
    let arguments: Vec<String> = env::args().collect();

    let mut br: Box<dyn BufRead> = if atty::is(atty::Stream::Stdin) {
        let f = File::open(&arguments[1])?;
        Box::new(BufReader::new(f))
    } else {
        Box::new(BufReader::new(std::io::stdin()))
    };

    // For each message
    loop {
        let buf = br.fill_buf()?;
        if buf.len() == 0 {
            break;
        };

        // Standard header
        let sh = read_standard_header(&mut br)?;

        // Content
        let mut content_buf = vec![0u8; sh.body_length];
        br.read_exact(&mut content_buf)?;
        while !content_buf.is_empty() {
            let field = get_field(&content_buf);
            let tv = field_to_tag_value(field);

            match tv.0 {
                [0x033, 0x035] => print!(
                    "{}{}{}",
                    "35=".truecolor(80, 80, 80),
                    std::str::from_utf8(tv.1).unwrap().yellow(),
                    "|".truecolor(80, 80, 80),
                ),
                _ => print!(
                    "{}{}{}{}",
                    std::str::from_utf8(tv.0).unwrap().truecolor(80, 80, 80),
                    "=".truecolor(80, 80, 80),
                    std::str::from_utf8(tv.1).unwrap().white(),
                    "|".truecolor(80, 80, 80),
                ),
            }
            content_buf = content_buf[field.len() + 1..].to_vec();
        }

        // Checksum
        let _crc = read_standard_trailer(&mut br)?;
        println!("{}", "CRC".truecolor(0, 100, 0));
    }

    Ok(())
}
