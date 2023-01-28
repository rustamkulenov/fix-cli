use colored::*;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use fixcat::*;

const MAX_MSG_LEN: usize = 1024 * 1024 * 5; // 5Mb

fn main() -> std::io::Result<()> {
    let arguments: Vec<String> = env::args().collect();

    let mut br: Box<dyn BufRead> = if atty::is(atty::Stream::Stdin) {
        let f = File::open(&arguments[1])?;
        Box::new(BufReader::new(f))
    } else {
        Box::new(BufReader::new(std::io::stdin()))
    };

    let mut content_buf = vec![0u8; MAX_MSG_LEN];

    // For each message
    loop {
        let buf = br.fill_buf()?;
        if buf.len() == 0 {
            break;
        };

        // Standard header
        let sh = read_standard_header(&mut br)?;

        // Content
        //let mut content_buf = vec![0u8; sh.body_length];
        let content_slice = &mut content_buf[0..sh.body_length];
        br.read_exact(content_slice)?;
        let mut start = 0;

        while start < sh.body_length {
            let field = get_field(&content_slice[start..]);
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
        let _crc = read_standard_trailer(&mut br)?;
        println!("{}", "CRC".truecolor(0, 100, 0));
    }

    Ok(())
}
