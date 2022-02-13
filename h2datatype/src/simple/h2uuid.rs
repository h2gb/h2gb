use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use uuid::{Uuid, Version};

use generic_number::{Context, Endian};
use crate::{Alignment, Data, H2Type, H2Types, H2TypeTrait};

/// Defines a UUID.
///
/// An UUID address is always represented as a 16-byte value. It's always
/// displayed in standard UUID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2UUID {
    endian: Endian,
    include_version: bool,
}

impl H2UUID {
    pub fn new_aligned(alignment: Alignment, endian: Endian, include_version: bool) -> H2Type {
        H2Type::new(alignment, H2Types::H2UUID(Self {
            endian: endian,
            include_version: include_version,
        }))
    }

    pub fn new(endian: Endian, include_version: bool) -> H2Type {
        Self::new_aligned(Alignment::None, endian, include_version)
    }
}

impl H2TypeTrait for H2UUID {
    fn base_size(&self, _context: Context, _data: &Data) -> SimpleResult<usize> {
        Ok(16)
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        let number = context.read_u128(self.endian)?;
        let uuid = Uuid::from_u128(number);

        if self.include_version {
            Ok(match uuid.get_version() {
                Some(Version::Nil)    => format!("{} (Nil UUID)",        uuid),
                Some(Version::Mac)    => format!("{} (UUIDv1 / MAC)",    uuid),
                Some(Version::Dce)    => format!("{} (UUIDv2 / DCE)",    uuid),
                Some(Version::Md5)    => format!("{} (UUIDv3 / MD5)",    uuid),
                Some(Version::Random) => format!("{} (UUIDv4 / Random)", uuid),
                Some(Version::Sha1)   => format!("{} (UUIDv5 / SHA1)"  , uuid),
                None                  => format!("{} (Invalid UUID)",    uuid),
            })
        } else {
            Ok(uuid.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use generic_number::Context;

    #[test]
    fn test_uuid() -> SimpleResult<()> {
        let tests = vec![
            (
                b"\x85\xf8\x6f\xca\x66\x8e\x11\xec\x90\xd6\x02\x42\xac\x12\x00\x03".to_vec(),
                "85f86fca-668e-11ec-90d6-0242ac120003",
                "UUIDv1 / MAC",
            ),

            (
                b"\x45\xa1\x13\xac\xc7\xf2\x30\xb0\x90\xa5\xa3\x99\xab\x91\x27\x16".to_vec(),
                "45a113ac-c7f2-30b0-90a5-a399ab912716",
                "UUIDv3 / MD5",
            ),

            (
                b"\x29\x5c\xf0\x7f\xeb\xf2\x4d\x87\xa8\x1c\x0f\x64\xa0\xe2\xe0\x2f".to_vec(),
                "295cf07f-ebf2-4d87-a81c-0f64a0e2e02f",
                "UUIDv4 / Random",
            ),

            (
                b"\x4b\xe0\x64\x3f\x1d\x98\x57\x3b\x97\xcd\xca\x98\xa6\x53\x47\xdd".to_vec(),
                "4be0643f-1d98-573b-97cd-ca98a65347dd",
                "UUIDv5 / SHA1",
            ),

            (
                b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),
                "00000000-0000-0000-0000-000000000000",
                "Nil UUID",
            ),

            (
                b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01".to_vec(),
                "00000000-0000-0000-0000-000000000001",
                "Invalid UUID",
            ),
        ];

        for (data, uuid, version) in tests {
            let context = Context::new(&data);

            // Don't display version
            assert_eq!(uuid, H2UUID::new(Endian::Big, false).to_display(context, &Data::default())?);

            // Do display version
            assert_eq!(format!("{} ({})", uuid, version), H2UUID::new(Endian::Big, true).to_display(context, &Data::default())?);
        }

        Ok(())
    }
}
