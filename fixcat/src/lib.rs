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
    pub fn new(begin_string: &'a str, msg_type: &'a str, body_length: usize) -> Self {
        Self {
            begin_string,
            body_length,
            msg_type,
            secure_data_len: None,
            message_encoding: None,
        }
    }
}

pub fn read_standard_header(
    br: &mut dyn BufRead,
    field_separator: u8,
) -> std::io::Result<(StandardHeader, usize)> {
    let buf = br.fill_buf()?;
    if buf.len() == 0 {
        ()
    };

    let mut start: usize = 0;

    // BeginString(8)
    let token = get_field(&buf[start..], field_separator);
    start += token.len() + 1;
    let begin_string = field_to_tag_value(token);
    assert!(begin_string.0 == [0x038]); // BeginString(8)
    let begin_string = std::str::from_utf8(begin_string.1).unwrap();

    // BodyLength(9)
    let token = get_field(&buf[start..], field_separator);
    start += token.len() + 1;
    let body_length = field_to_tag_value(token);
    assert!(body_length.0 == [0x039]); // BodyLength(9)
    let body_length = std::str::from_utf8(body_length.1).unwrap();
    let body_length: usize = body_length.parse().unwrap();

    // Will consume only 2 first tags to match msg size with BodyLength(9) tag value.
    // Next read of br will also contain MsgType(35) tag
    let consume_amt = start;

    // MsgType(35)
    let token = get_field(&buf[start..], field_separator);
    let msg_type = field_to_tag_value(token);
    assert!(msg_type.0 == [0x033, 0x035]); // TAG_MSG_TYPE
    let msg_type = std::str::from_utf8(msg_type.1).unwrap();

    let sh = StandardHeader::new(begin_string, msg_type, body_length);

    Ok((sh, consume_amt))
}

pub fn read_standard_trailer(
    br: &mut dyn BufRead,
    field_separator: u8,
) -> std::io::Result<(StandardTrailer, usize)> {
    let buf = br.fill_buf()?;
    if buf.len() == 0 {
        ()
    };

    let token = get_field(buf, field_separator);
    let crc = field_to_tag_value(token);
    assert!(crc.0 == [0x031, 0x030]); // CheckSum(10)
    let crc_value = std::str::from_utf8(crc.1).unwrap();

    let amt = token.len() + 1;

    Ok((StandardTrailer::new(crc_value), amt))
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
    assert!(buf.len() > 0);

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
