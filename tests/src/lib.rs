use num_bigint::BigInt;
use std::collections::HashMap;

witnesscalc_adapter::witness!(multiplier2);

pub fn create_witness(inputs: HashMap<String, Vec<BigInt>>, dat_file_path: &str) -> Vec<BigInt> {
    multiplier2_witness(inputs, dat_file_path)
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use num_bigint::BigInt;

    use crate::create_witness;

    #[test]
    fn test_witnesscalc() {
        let mut inputs = HashMap::new();

        let a = BigInt::from(2u8);
        let b = BigInt::from(3u8);

        inputs.insert("a".to_string(), vec![a]);
        inputs.insert("b".to_string(), vec![b]);

        let result = create_witness(inputs, "./testdata/multiplier2.dat");

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], BigInt::from(1u8));
        assert_eq!(
            result[1],
            BigInt::from(
                6u8 // 2 x 3
            )
        );
        assert_eq!(result[2], BigInt::from(2u8));
        assert_eq!(result[3], BigInt::from(3u8));
    }
}
