use std::{collections::HashMap, io};

use num_bigint::BigInt;
use num_traits::FromBytes;

pub fn parse_witness_to<T>(
    buffer: &[u8],
    from_le_bytes: impl Fn(&[u8]) -> T,
) -> io::Result<Vec<T>> {
    /// Prints to the standard ouput only in debug build.
    macro_rules! debug_println {
        ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
    }

    let mut pos = 0;

    // skip format bytes (4 bytes)
    // ensure that this says "wtns" in ASCII
    if &buffer[pos..pos + 4] != b"wtns" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid witness file format.",
        ));
    }
    pos += 4;

    // read version (4 bytes)
    let _version = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
    debug_println!("version: {}", _version);
    pos += 4;

    // read number of sections (4 bytes)
    let n_sections = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
    debug_println!("n_sections: {}", n_sections);
    pos += 4;

    // number of 8 bit integers per field element
    let mut n8 = 0;

    // iterate through sections to find section_id = 2 (witness data)
    //
    // each [section]
    // - section id (4 bytes)
    // - section length (8 bytes)
    for _ in 0..n_sections {
        let section_id = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
        debug_println!("section_id: {}", section_id);
        pos += 4;

        let section_length = u64::from_le_bytes(buffer[pos..pos + 8].try_into().unwrap());
        debug_println!("section_length: {}", section_length);
        pos += 8;

        match section_id {
            // [section 1]
            // - `n8` number of 8 bit integers per field element (4 bytes / u32)
            // - the field `q` value (32 bytes)
            // - number of witness values (4 bytes / `u32`)
            1 => {
                n8 = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
                debug_println!("n8: {}", n8);
                pos += 4;

                let _q = BigInt::from_signed_bytes_le(&buffer[pos..pos + 32]);
                debug_println!("q: {}", _q);
                pos += 32;

                let _n_witness_values =
                    u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
                debug_println!("n_witness_values: {}", _n_witness_values);
                pos += 4;
            }

            // [section 2]
            // - witness data (`n8` bytes per element, `section_length` bytes total)
            2 => {
                // read & convert witness bytes chunk by chunk
                let elements: Vec<T> = buffer[pos..pos + section_length as usize]
                    .chunks(usize::try_from(n8).unwrap())
                    .map(from_le_bytes)
                    .collect();

                return Ok(elements);
            }
            // skip any other section
            _ => {
                pos += section_length as usize;
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Witness section not found.",
    ))
}

#[inline]
pub fn parse_witness_to_bigints(buffer: &[u8]) -> io::Result<Vec<BigInt>> {
    parse_witness_to(buffer, BigInt::from_le_bytes)
}

pub fn convert_inputs_to_json(inputs: HashMap<String, Vec<String>>) -> String {
    //Convert the inputs into a JSON string
    let json_map: serde_json::Map<String, serde_json::Value> = inputs
        .into_iter()
        .map(|(key, values)| (key, serde_json::Value::from(values)))
        .collect();
    let json = serde_json::Value::Object(json_map);
    serde_json::to_string(&json).unwrap()
}
