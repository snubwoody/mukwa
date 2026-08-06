// Mukwa - Personal finance
// Copyright (C) 2026  Wakunguma Kalimukwa
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ParseError {
    /// The `OFXHEADER` attribute had an invalid value.
    InvalidOfxHeader,
    /// The `VERSION` attribute had an invalid value.
    InvalidVersion,
    /// An attribute had an invalid value.
    InvalidAttribute { attr: String, value: String },
}

impl ParseError {
    fn invalid_attr(attr: &str, value: &str) -> ParseError {
        ParseError::InvalidAttribute {
            attr: attr.to_owned(),
            value: value.to_owned(),
        }
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidOfxHeader => write!(f, "Invalid OFX header"),
            ParseError::InvalidVersion => write!(f, "Invalid version"),
            ParseError::InvalidAttribute { attr, value } => {
                write!(f, "The {attr} attribute has an invalid value: {value}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Security {
    #[default]
    None,
    Type1,
}

impl FromStr for Security {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONE" => Ok(Security::None),
            "TYPE1" => Ok(Security::Type1),
            _ => Err(ParseError::invalid_attr("SECURITY", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Encoding {
    #[default]
    UsAscii,
    Unicode,
}

impl FromStr for Encoding {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "UNICODE" => Ok(Encoding::Unicode),
            "USASCII" => Ok(Encoding::UsAscii),
            _ => Err(ParseError::invalid_attr("ENCODING", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Header {
    /// The version number of the OFX declaration.
    ofx_header: u32,
    data: String,
    security: Security,
    encoding: Encoding,
    /// The version number of the OFX data block.
    version: u32,
    charset: u32,
    compression: String,
    old_file_uid: String,
    new_file_uid: String,
}

pub struct Ofx {
    header: Header,
}

pub struct Body {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct SignOnResponse {
    status: String,
    /// The date and time of the server response
    date_time: String,
    /// The language used in text responses
    language: String,
}

fn parse_header(input: &str) -> Result<Header, ParseError> {
    // TODO: parse out of order headers
    let lines = input
        .lines()
        .map(|line| line.split(":").collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let ofx_header = lines[0][1]
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidOfxHeader)?;
    let data = lines[1][1].to_owned();
    let version = lines[2][1]
        .parse::<u32>()
        .map_err(|_| ParseError::InvalidVersion)?;
    let security = lines[3][1].parse::<Security>()?;
    let encoding = lines[4][1].parse::<Encoding>()?;
    let charset = lines[5][1]
        .parse::<u32>()
        .map_err(|_| ParseError::invalid_attr("ENCODING", lines[5][1]))?;
    let compression = lines[6][1].to_owned();
    let old_file_uid = lines[7][1].to_owned();
    let new_file_uid = lines[8][1].to_owned();

    let header = Header {
        ofx_header,
        compression,
        charset,
        version,
        data,
        new_file_uid,
        old_file_uid,
        encoding,
        security,
    };

    Ok(header)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_header() -> crate::Result<()> {
        let mut data = String::new();
        data += "OFXHEADER:100\n";
        data += "DATA:OFXSGML\n";
        data += "VERSION:102\n";
        data += "SECURITY:TYPE1\n";
        data += "ENCODING:USASCII\n";
        data += "CHARSET:1000\n";
        data += "COMPRESSION:NONE\n";
        data += "OLDFILEUID:NONE\n";
        data += "NEWFILEUID:NONE";

        let header = super::parse_header(&data).unwrap();
        let expected = Header {
            ofx_header: 100,
            data: String::from("OFXSGML"),
            version: 102,
            security: Security::Type1,
            encoding: Encoding::UsAscii,
            charset: 1000,
            compression: String::from("NONE"),
            old_file_uid: String::from("NONE"),
            new_file_uid: String::from("NONE"),
        };
        assert_eq!(header, expected);
        Ok(())
    }
}
