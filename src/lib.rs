pub mod quickfix_spec;
mod tests;

use std::io::prelude::*;

//https://www.fixtrading.org/standards/tagvalue-online/

pub const TAG_DELIMETER: u8 = 0x03d; // '='
pub const SOH: u8 = 0x01;
pub const PIPE: u8 = 0x7c; // '|'

pub struct StandardHeader<'a> {
    pub begin_string: &'a str,             // 8
    pub body_length: usize,                // 9
    pub msg_type: &'a str,                 // 35
    pub secure_data_len: Option<usize>,    // 90
    pub message_encoding: Option<&'a str>, // 347
}

pub struct StandardTrailer<'a> {
    pub check_sum: &'a str,           // 10
    pub signature_len: Option<usize>, // 93
    pub signature: Option<&'a str>,   // 89
}

impl<'a> StandardTrailer<'a> {
    pub fn new(check_sum: &'a str) -> Self {
        Self {
            check_sum,
            signature_len: None,
            signature: None,
        }
    }
}

impl<'a> StandardHeader<'a> {
    pub fn new(begin_string: &'a str, body_length: usize) -> Self {
        Self {
            begin_string,
            body_length,
            msg_type: "",
            secure_data_len: None,
            message_encoding: None,
        }
    }
}

pub fn read_header<'a>(
    br: &mut dyn BufRead,
    field_separator: u8,
    vec: &'a mut Vec<u8>,
) -> std::io::Result<StandardHeader<'a>> {
    let p1 = br.read_until(field_separator, vec).unwrap();
    let p2 = br.read_until(field_separator, vec).unwrap() + p1;

    let begin_string = field_to_tag_value(&vec[..p1 - 1]);
    assert!(begin_string.0 == [0x038]); // BeginString(8)
    let begin_string = std::str::from_utf8(begin_string.1).unwrap();

    let body_length = field_to_tag_value(&vec[p1..p2 - 1]);
    assert!(body_length.0 == [0x039]); // BodyLength(9)
    let body_length = std::str::from_utf8(body_length.1).unwrap();
    let body_length: usize = body_length.parse().unwrap();

    let sh: StandardHeader = StandardHeader::new(begin_string, body_length);
    Ok(sh)
}

pub fn read_crc(br: &mut dyn BufRead, field_separator: u8) -> () {
    let mut trailer: [u8; 7] = [0; 7]; // Read CRC with ending SOH
    br.read_exact(&mut trailer).unwrap();
}

pub fn get_field(buf: &[u8], field_separator: u8) -> &[u8] {
    let mut len: usize = 0;
    for v in buf {
        if *v == field_separator {
            break;
        }
        len += 1;
    }
    if len == buf.len() {
        &[] // field without separator
    } else {
        &buf[0..len]
    }
}

pub fn get_field_as_string(buf: &[u8], field_separator: u8) -> &str {
    let f = get_field(buf, field_separator);
    let s = std::str::from_utf8(f).unwrap();
    s
}

/*
A well-formed field has the form:

tag=value<SOH>

A field shall be considered malformed if any of the following occurs as a result of encoding:

    the tag is empty
    the tag delimiter is missing
    the value is empty
    the value contains an <SOH> character and the datatype of the field is not data or XMLdata
    the datatype of the field is data and the field is not immediately preceded by its associated Length field.
*/
pub fn field_to_tag_value(buf: &[u8]) -> (&[u8], &[u8]) {
    assert!(
        buf.len() > 0,
        "Can not read tag and value from empty field buffer"
    );

    let mut i: usize = 0;
    for v in buf {
        if *v == TAG_DELIMETER {
            break;
        }
        i += 1;
    }

    assert!(i < buf.len() - 1);
    (&buf[0..i], &buf[i + 1..])
}
