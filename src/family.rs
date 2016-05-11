use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Error, ErrorKind, Read, Write};

use super::element::IconElement;

/// The first four bytes of an ICNS file:
const ICNS_MAGIC_LITERAL: &'static [u8; 4] = b"icns";

/// The length of an icon family header, in bytes:
const ICON_FAMILY_HEADER_LENGTH: u32 = 8;

/// A set of icons stored in a single ICNS file.
pub struct IconFamily {
    /// The icon elements stored in the ICNS file.
    pub elements: Vec<IconElement>,
}

impl IconFamily {
    /// Creates a new, empty icon family.
    pub fn new() -> IconFamily {
        IconFamily { elements: Vec::new() }
    }

    /// Returns the encoded length of the file, in bytes, including the
    /// length of the header.
    pub fn total_length(&self) -> u32 {
        let mut length = ICON_FAMILY_HEADER_LENGTH;
        for element in &self.elements {
            length += element.total_length();
        }
        length
    }

    /// Reads an icon family from an ICNS file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<IconFamily> {
        let mut magic = [0u8; 4];
        try!(reader.read_exact(&mut magic));
        if magic != *ICNS_MAGIC_LITERAL {
            let msg = "not an icns file (wrong magic literal)";
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
        let file_length = try!(reader.read_u32::<BigEndian>());
        let mut file_position: u32 = ICON_FAMILY_HEADER_LENGTH;
        let mut family = IconFamily::new();
        while file_position < file_length {
            let element = try!(IconElement::read(reader.by_ref()));
            file_position += element.total_length();
            family.elements.push(element);
        }
        Ok(family)
    }

    /// Writes the icon family to an ICNS file.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        try!(writer.write_all(ICNS_MAGIC_LITERAL));
        try!(writer.write_u32::<BigEndian>(self.total_length()));
        for element in &self.elements {
            try!(element.write(writer.by_ref()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::element::IconElement;
    use super::super::icontype::OSType;
    use std::io::{Cursor, Read};

    #[test]
    fn write_empty_icon_family() {
        let family = IconFamily::new();
        assert_eq!(0, family.elements.len());
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x08", &output as &[u8]);
    }

    #[test]
    fn read_icon_family_with_fake_elements() {
        let input: Cursor<&[u8]> =
            Cursor::new(b"icns\0\0\0\x1fquux\0\0\0\x0efoobarbaz!\0\0\0\x09#");
        let family = IconFamily::read(input).expect("read failed");
        assert_eq!(2, family.elements.len());
        assert_eq!(OSType(*b"quux"), family.elements[0].ostype);
        assert_eq!(6, family.elements[0].data.len());
        assert_eq!(OSType(*b"baz!"), family.elements[1].ostype);
        assert_eq!(1, family.elements[1].data.len());
    }

    #[test]
    fn write_icon_family_with_fake_elements() {
        let mut family = IconFamily::new();
        family.elements
              .push(IconElement::new(OSType(*b"quux"), b"foobar".to_vec()));
        family.elements
              .push(IconElement::new(OSType(*b"baz!"), b"#".to_vec()));
        let mut output: Vec<u8> = vec![];
        family.write(&mut output).expect("write failed");
        assert_eq!(b"icns\0\0\0\x1fquux\0\0\0\x0efoobarbaz!\0\0\0\x09#",
                   &output as &[u8]);
    }
}
