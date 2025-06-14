#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use num_bigint::BigInt;
    use witnesscalc_adapter::{convert_inputs_to_json, parse_witness_to_bigints};

    witnesscalc_adapter::witness!(multiplier2);
    witnesscalc_adapter::witness!(keccak256_256_test);
    witnesscalc_adapter::witness!(rsa_main);

    #[test]
    fn test_witnesscalc() {
        let mut inputs = HashMap::new();

        let a = "2".to_string();
        let b = "3".to_string();

        inputs.insert("a".to_string(), vec![a]);
        inputs.insert("b".to_string(), vec![b]);

        let black_box_inputs = std::hint::black_box(inputs);
        let result = multiplier2_witness(&convert_inputs_to_json(black_box_inputs));
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

        let black_box_inputs = std::hint::black_box(inputs);
        use std::time::Instant;
        let start = Instant::now();
        let _ = keccak256_256_test_witness(&convert_inputs_to_json(black_box_inputs));
        let end = Instant::now();
        println!(
            "Time taken for keccak256_256_test: {:?}",
            end.duration_since(start)
        );
    }

    #[test]
    #[ignore = "unstable"]
    fn test_rsa_main_witnesscalc() {
        let json_input =
            std::fs::read_to_string("testdata/rsa_main.json").expect("Unable to read file");

        use std::time::Instant;
        let start = Instant::now();
        let _ = rsa_main_witness(&json_input);
        let end = Instant::now();
        println!("Time taken for rsa_main: {:?}", end.duration_since(start));
    }
}
