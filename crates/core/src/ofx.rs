// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

// OFXHEADER:100
// DATA:OFXSGML
// VERSION:102
// SECURITY:TYPE1
// ENCODING:USASCII
// CHARSET:1252
// COMPRESSION:NONE
// OLDFILEUID:NONE
// NEWFILEUID:NONE

use std::fmt::Formatter;
use std::iter::Peekable;

#[derive(Debug)]
pub struct ParseOfxError;

impl std::fmt::Display for ParseOfxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"Error parsing OFX data")
    }
}

impl std::error::Error for ParseOfxError{

}

#[derive(Default)]
pub struct Header {
    ofx_header: u8,
    version: u8
}

fn parse(input: &str) -> Header {
    let lines: Vec<&str> = input.lines().collect();
    let ofx_header: u8 = get_value(&lines,"OFXHEADER").unwrap().parse().unwrap();
    // let data = lines.next().unwrap();
    let version: u8 = get_value(&lines,"VERSION").unwrap().parse().unwrap();
    // let security = lines.next().unwrap();
    // let encoding = lines.next().unwrap();
    // let charset = lines.next().unwrap();
    // let compression = lines.next().unwrap();
    // let old_file_uid = lines.next().unwrap();
    // let new_file_uid = lines.next().unwrap();

    Header { ofx_header,version }
}

fn get_value(lines: &[&str],key: &str) -> Option<String>{
    for line in lines {
        if line.replace(" ","").starts_with(&format!("{key}:")){
            let parts: Vec<&str> = line.split(":").collect();
            return Some(parts[1].to_string())
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_header() {
        let data = "OFXHEADER:100\n
            DATA:OFXSGML\n
            VERSION:102\n
            SECURITY:TYPE1\n
            ENCODING:USASCII\n
            CHARSET:1252\n
            COMPRESSION:NONE\n
            OLDFILEUID:NONE\n
            NEWFILEUID:NONE\n";

        let header = parse(data);
        assert_eq!(header.ofx_header, 100);
        assert_eq!(header.version, 102);
    }
}
