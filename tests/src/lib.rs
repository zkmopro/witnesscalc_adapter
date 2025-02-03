use anyhow::Result;
use std::collections::HashMap;
use witnesscalc_adapter::*;

witnesscalc_adapter::witness!(multiplier2);
witnesscalc_adapter::witness!(keccak256_256_test);
witnesscalc_adapter::witness!(rsa_main);

pub fn create_witness(inputs: HashMap<String, Vec<String>>) -> Result<Vec<u8>> {
    multiplier2_witness(&convert_inputs_to_json(inputs))
}

pub fn create_keccak256_256_test_witness(inputs: HashMap<String, Vec<String>>) -> Result<Vec<u8>> {
    keccak256_256_test_witness(&convert_inputs_to_json(inputs))
}

pub fn create_rsa_main_witness(json_input: String) -> Result<Vec<u8>> {
    rsa_main_witness(&json_input)
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use num_bigint::BigInt;
    use witnesscalc_adapter::parse_witness_to_bigints;

    use crate::create_keccak256_256_test_witness;
    use crate::create_rsa_main_witness;
    use crate::create_witness;
    #[test]
    fn test_witnesscalc() {
        let mut inputs = HashMap::new();

        let a = "2".to_string();
        let b = "3".to_string();

        inputs.insert("a".to_string(), vec![a]);
        inputs.insert("b".to_string(), vec![b]);

        let result = create_witness(inputs);
        assert!(result.is_ok());
        let witness_bytes = result.unwrap();
        let witness = parse_witness_to_bigints(&witness_bytes).unwrap();

        assert_eq!(witness.len(), 4);
        assert_eq!(witness[0], BigInt::from(1u8));
        assert_eq!(witness[1], BigInt::from(6u8));
        assert_eq!(witness[2], BigInt::from(2u8));
        assert_eq!(witness[3], BigInt::from(3u8));
    }

    #[test]
    fn test_keccak256_256_test_witnesscalc() {
        let mut inputs = HashMap::new();

        inputs.insert("in".to_string(), vec![0u8.to_string(); 256]);

        use std::time::Instant;
        let start = Instant::now();
        let _ = create_keccak256_256_test_witness(inputs);
        let end = Instant::now();
        println!(
            "Time taken for keccak256_256_test: {:?}",
            end.duration_since(start)
        );
    }

    #[test]
    fn test_rsa_main_witnesscalc() {
        let json_input =
            std::fs::read_to_string("testdata/rsa_main.json").expect("Unable to read file");

        use std::time::Instant;
        let start = Instant::now();
        let _ = create_rsa_main_witness(json_input);
        let end = Instant::now();
        println!("Time taken for rsa_main: {:?}", end.duration_since(start));
    }
}
