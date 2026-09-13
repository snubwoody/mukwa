// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Wakunguma Kalimukwa

use std::fmt::{Display, Formatter};
use std::str::FromStr;

// OFXHEADER:100
// DATA:OFXSGML
// VERSION:102
// SECURITY:TYPE1
// ENCODING:USASCII
// CHARSET:1252
// COMPRESSION:NONE
// OLDFILEUID:NONE
// NEWFILEUID:NONE

#[derive(Debug)]
pub enum ParseOfxError{
    InvalidHeader
}

impl Display for ParseOfxError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseOfxError::InvalidHeader => write!(f,"Invalid header")
        }
    }
}

impl std::error::Error for ParseOfxError{}

#[derive(Copy, Clone,PartialOrd, PartialEq,Ord, Eq,Debug)]
pub enum Data{
    OFXSGML
}

impl FromStr for Data{
    type Err = ParseOfxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OFXSGML" => Ok(Data::OFXSGML),
            _ => Err(ParseOfxError::InvalidHeader)
        }
    }
}

#[derive(Copy, Clone,PartialOrd, PartialEq,Ord, Eq,Debug)]
pub enum Security{
    None,
    Type1
}

impl FromStr for Security{
    type Err = ParseOfxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TYPE1" => Ok(Security::Type1),
            "NONE" => Ok(Security::None),
            _ => Err(ParseOfxError::InvalidHeader)
        }
    }
}

#[derive(Copy, Clone,PartialOrd, PartialEq,Ord, Eq,Debug)]
pub enum Encoding{
    USAscii,
    Utf8
}

impl FromStr for Encoding{
    type Err = ParseOfxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USASCII" => Ok(Encoding::USAscii),
            "UTF-8" => Ok(Encoding::Utf8),
            _ => Err(ParseOfxError::InvalidHeader)
        }
    }
}

#[derive(Copy, Clone,PartialOrd, PartialEq,Ord, Eq,Debug)]
pub enum Charset{
    Latin1,
    WindowsLatin1,
    None
}

impl FromStr for Charset{
    type Err = ParseOfxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ISO-8859-1" => Ok(Charset::Latin1),
            "1252" => Ok(Charset::WindowsLatin1),
            "NONE" => Ok(Charset::None),
            _ => Err(ParseOfxError::InvalidHeader)
        }
    }
}

#[derive(Copy, Clone,PartialOrd, PartialEq,Ord, Eq,Debug)]
pub enum Compression{
    None
}

impl FromStr for Compression{
    type Err = ParseOfxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONE" => Ok(Compression::None),
            _ => Err(ParseOfxError::InvalidHeader)
        }
    }
}

pub struct OfxHeader{
    /// `OFXHEADER` specifies the version number of the Open Financial Exchange headers.
    ///
    /// The `OFXHEADER` value change its major number only if an existing client is unable to process
    /// the new header. This can occur because of a complete syntax change in a header, or a significant
    /// change in the semantics of an existing header element.
    ///
    /// The current version of the Open Financial Exchange headers is version 1.0 (`OFXHEADER`:100).
    ofxheader: u8,
    /// DATA specifies the content type, in this case OFXSGML.
    ///
    /// The value of the DATA element changes only when an entirely new syntax is introduced. In the
    /// case of OFXSGML, a new syntax would have to be non-SGML compliant to warrant a new
    /// DATA value. It is possible that there will be more than one syntax in use at the same time to meet
    /// different needs.
    data: Data,
    /// VERSION specifies the version number of the Document Type Definition (DTD) used for
    /// parsing.
    ///
    /// The current accepted values for VERSION are 102, 151, and 160.
    version: u8,
    /// `SECURITY` defines the type of application-level security, if any, that is used for the <OFX> block.
    /// The values for `SECURITY` can be NONE or TYPE1.
    security: Security,
    /// `ENCODING` defines the text encoding used for character data. The values for ENCODING can
    /// be USASCII or UTF-8.
    encoding: Encoding,
    /// CHARSET defines the character set used for character data. The values for CHARSET may be
    /// ISO-8859-1 (Latin-1), 1252 (Windows Latin-1), or NONE. Any value specified here is likely to be
    /// ignored by an OFX client or server
    charset: Charset,
    compression: Compression,
    /// OLDFILEUID is used together with NEWFILEUID only when the client and server support file-
    /// based error recovery. OLDFILEUID identifies the last request and response that was received
    /// and processed by the client.
    old_file_uid: Option<String>,
    /// NEWFILEUID uniquely identifies this request file. The NEWFILEUID, which clients must send
    /// with every request file and which servers must echo in the response, serves two purposes:
    new_file_uid: Option<String>,
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn charset_from_str(){
        assert_eq!(Charset::from_str("1252").unwrap(),Charset::WindowsLatin1);
        assert_eq!(Charset::from_str("ISO-8859-1").unwrap(),Charset::Latin1);
        assert_eq!(Charset::from_str("NONE").unwrap(),Charset::None);
        assert!(Charset::from_str("invalid").is_err());
    }

    #[test]
    fn encoding_from_str(){
        assert_eq!(Encoding::from_str("USASCII").unwrap(),Encoding::USAscii);
        assert_eq!(Encoding::from_str("UTF-8").unwrap(),Encoding::Utf8);
        assert!(Encoding::from_str("invalid").is_err());
    }
}