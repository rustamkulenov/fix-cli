mod tests;

use std::io::prelude::*;

//https://www.fixtrading.org/standards/tagvalue-online/

pub const TAG_DELIMETER: u8 = 0x03d; // '='
pub const SOH: u8 = 0x01;
pub const PIPE: u8 = 0x7c; // '|'

pub struct StandardHeader {
    pub begin_string: String,             // 8
    pub body_length: usize,               // 9
    pub msg_type: String,                 // 35
    pub secure_data_len: Option<usize>,   // 90
    pub message_encoding: Option<String>, // 347
}

pub struct StandardTrailer {
    pub check_sum: String,            // 10
    pub signature_len: Option<usize>, // 93
    pub signature: Option<String>,    // 89
}

impl StandardTrailer {
    pub fn new(check_sum: String) -> Self {
        Self {
            check_sum,
            signature_len: None,
            signature: None,
        }
    }
}

impl StandardHeader {
    pub fn new() -> Self {
        Self {
            begin_string: String::from(""),
            body_length: 0,
            msg_type: String::from(""),
            secure_data_len: None,
            message_encoding: None,
        }
    }
}

pub fn read_standard_header(
    br: &mut dyn BufRead,
    field_separator: u8,
) -> std::io::Result<StandardHeader> {
    let buf = br.fill_buf()?;
    if buf.len() == 0 {
        ()
    };

    let mut start: usize = 0;
    let mut sh = StandardHeader::new();

    // BeginString(8)
    let token = get_field(&buf[start..], field_separator);
    start += token.len() + 1;
    let begin_string = field_to_tag_value(token);
    let value = String::from_utf8(begin_string.1.to_vec()).unwrap();
    assert!(begin_string.0 == [0x038]); // BeginString(8)
    sh.begin_string = value;

    // BodyLength(9)
    let token = get_field(&buf[start..], field_separator);
    start += token.len() + 1;
    let body_length = field_to_tag_value(token);
    let value = String::from_utf8(body_length.1.to_vec()).unwrap();
    assert!(body_length.0 == [0x039]); // BodyLength(9)
    sh.body_length = value.parse().unwrap();

    // Will consume only 2 first tags to match msg size with BodyLength(9) tag value.
    // Next read of br will also contain MsgType(35) tag
    let consume_amt = start;

    // MsgType(35)
    let token = get_field(&buf[start..], field_separator);
    let msg_type = field_to_tag_value(token);
    let value = String::from_utf8(msg_type.1.to_vec()).unwrap();
    assert!(msg_type.0 == [0x033, 0x035]); // TAG_MSG_TYPE
    sh.msg_type = value;

    br.consume(consume_amt);

    Ok(sh)
}

pub fn read_standard_trailer(
    br: &mut dyn BufRead,
    field_separator: u8,
) -> std::io::Result<StandardTrailer> {
    let buf = br.fill_buf()?;
    if buf.len() == 0 {
        ()
    };

    let token = get_field(buf, field_separator);
    let crc = field_to_tag_value(token);
    assert!(crc.0 == [0x031, 0x030]); // CheckSum(10)
    let crc_value = String::from_utf8(crc.1.to_vec()).unwrap();

    let amt = token.len() + 1;
    br.consume(amt);

    Ok(StandardTrailer::new(crc_value))
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
