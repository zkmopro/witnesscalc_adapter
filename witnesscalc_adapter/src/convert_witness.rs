use std::io;

use num_bigint::BigInt;
use num_traits::FromBytes;

pub fn parse_witness_to_bigints(buffer: &[u8]) -> io::Result<Vec<BigInt>> {
    let mut pos = 0;

    // Skip the format bytes (4 bytes)
    pos += 4;

    // Read version (4 bytes)
    let _version = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
    println!("version: {:?}", _version);
    pos += 4;

    // Read number of sections (4 bytes)
    let n_sections = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
    pos += 4;

    println!("n_sections: {:?}", n_sections);

    // Iterate through sections to find section_id = 2 (witness data)
    let mut n8 = 0;
    for _ in 0..n_sections {
        let section_id = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
        println!("section_id: {:?}", section_id);
        pos += 4;

        let section_length = u64::from_le_bytes(buffer[pos..pos + 8].try_into().unwrap());
        println!("section_length: {:?}", section_length);
        pos += 8;

        //for section 1: section id (4 bytes), section length (8 bytes), n8 number of 8 bit integers per field element (4 bytes), the field q value (32 bytes), and the number of witness values (4 bytes)
        if section_id == 1 {
            n8 = u32::from_le_bytes(buffer[pos..pos + 4].try_into().unwrap());
            println!("n8: {:?}", n8);
            let q = BigInt::from_signed_bytes_le(&buffer[pos + 4..pos + 36]);
            println!("q: {:?}", q);
            let n_witness_values =
                u32::from_le_bytes(buffer[pos + 36..pos + 40].try_into().unwrap());
            println!("n_witness_values: {:?}", n_witness_values);
        }

        if section_id == 2 {
            //count nonzero chunks
            // Witness data found, read section_length bytes
            let witness_data = &buffer[pos..pos + section_length as usize];

            // Convert witness data into BigInts (n8 bytes per BigInt)
            let bigints: Vec<BigInt> = witness_data
                .chunks(usize::try_from(n8).unwrap())
                .map(|chunk| BigInt::from_le_bytes(chunk))
                .collect();
            return Ok(bigints);
        } else {
            // Skip this section
            pos = pos + section_length as usize;
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "Witness section not found.",
    ))
}
