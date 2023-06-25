use colored::*;
use std::env;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;
use std::io::BufReader;

use fixcat::*;

const INITIAL_CONTENT_SIZE: usize = 1024;

fn main() -> std::io::Result<()> {
    let arguments: Vec<String> = env::args().collect();

    let mut br: Box<dyn BufRead> = if atty::is(atty::Stream::Stdin) {
        if arguments.len() < 2 {
            return Err(Error::new(std::io::ErrorKind::InvalidInput, "Provide file as a first parameter or stdin stream."))
        }
        let f = File::open(&arguments[1])?;
        Box::new(BufReader::new(f))
    } else {
        Box::new(BufReader::with_capacity(10, std::io::stdin()))
    };

    let mut content_buf = vec![0u8; INITIAL_CONTENT_SIZE];

    // For each message
    loop {
        let buf = br.fill_buf()?;
        if buf.len() == 0 {
            break;
        };

        // Standard header
        let (sh, consume_amt) = read_standard_header(&mut br, SOH)?;
        let body_length = sh.body_length;
        br.consume(consume_amt);

        // Content
        if content_buf.len() < body_length {
            // Resize buffer
            content_buf = vec![0u8; body_length];
        }
        let content_slice = &mut content_buf[0..body_length];
        br.read_exact(content_slice)?;
        let mut start = 0;

        while start < body_length {
            let field = get_field(&content_slice[start..], SOH);
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

            start += field.len() + 1;
        }

        // Checksum
        let (_crc, consume_amt) = read_standard_trailer(&mut br, SOH)?;
        br.consume(consume_amt);

        println!("{}", "CRC".truecolor(0, 100, 0));
    }

    Ok(())
}
