use anyhow::Result;
use num_bigint::BigInt;
use serde_json;
use std::collections::HashMap;

witnesscalc_adapter::witness!(multiplier2);
witnesscalc_adapter::witness!(keccak256_256_test);
witnesscalc_adapter::witness!(rsa_main);

pub fn convert_json(inputs: HashMap<String, Vec<BigInt>>) -> String {
    //Convert the inputs into a JSON string
    let json_map: serde_json::Map<String, serde_json::Value> = inputs
        .into_iter()
        .map(|(key, values)| {
            let values_as_strings: Vec<String> = values.iter().map(|num| num.to_string()).collect();
            (key, serde_json::Value::from(values_as_strings))
        })
        .collect();
    let json = serde_json::Value::Object(json_map);
    serde_json::to_string(&json).unwrap()
}

pub fn create_witness(inputs: HashMap<String, Vec<BigInt>>) -> Result<Vec<u8>> {
    multiplier2_witness(&convert_json(inputs))
}

pub fn create_keccak256_256_test_witness(inputs: HashMap<String, Vec<BigInt>>) -> Result<Vec<u8>> {
    keccak256_256_test_witness(&convert_json(inputs))
}

pub fn create_rsa_main_witness(inputs: HashMap<String, Vec<BigInt>>) -> Result<Vec<u8>> {
    rsa_main_witness(&convert_json(inputs))
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;
    use std::str::FromStr;

    use num_bigint::BigInt;
    use witnesscalc_adapter::parse_witness_to_bigints;

    use crate::create_keccak256_256_test_witness;
    use crate::create_rsa_main_witness;
    use crate::create_witness;
    #[test]
    fn test_witnesscalc() {
        let mut inputs = HashMap::new();

        let a = BigInt::from(2u8);
        let b = BigInt::from(3u8);

        inputs.insert("a".to_string(), vec![a]);
        inputs.insert("b".to_string(), vec![b]);

        let result = create_witness(inputs);
        assert!(result.is_ok());
        let witness_bytes = result.unwrap();
        let witness = parse_witness_to_bigints(&witness_bytes).unwrap();

        assert_eq!(witness.len(), 4);
        assert_eq!(witness[0], BigInt::from(1u8));
        assert_eq!(
            witness[1],
            BigInt::from(
                6u8 // 2 x 3
            )
        );
        assert_eq!(witness[2], BigInt::from(2u8));
        assert_eq!(witness[3], BigInt::from(3u8));
    }

    #[test]
    fn test_keccak256_256_test_witnesscalc() {
        let mut inputs = HashMap::new();

        inputs.insert("in".to_string(), vec![BigInt::from(0u8); 256]);

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
    #[ignore]
    fn test_rsa_main_witnesscalc() {
        let mut inputs = HashMap::new();

        inputs.insert(
            "signature".to_string(),
            vec![
                "3582320600048169363",
                "7163546589759624213",
                "18262551396327275695",
                "4479772254206047016",
                "1970274621151677644",
                "6547632513799968987",
                "921117808165172908",
                "7155116889028933260",
                "16769940396381196125",
                "17141182191056257954",
                "4376997046052607007",
                "17471823348423771450",
                "16282311012391954891",
                "70286524413490741",
                "1588836847166444745",
                "15693430141227594668",
                "13832254169115286697",
                "15936550641925323613",
                "323842208142565220",
                "6558662646882345749",
                "15268061661646212265",
                "14962976685717212593",
                "15773505053543368901",
                "9586594741348111792",
                "1455720481014374292",
                "13945813312010515080",
                "6352059456732816887",
                "17556873002865047035",
                "2412591065060484384",
                "11512123092407778330",
                "8499281165724578877",
                "12768005853882726493",
            ],
        );

        inputs.insert(
            "modulus".to_string(),
            vec![
                "13792647154200341559",
                "12773492180790982043",
                "13046321649363433702",
                "10174370803876824128",
                "7282572246071034406",
                "1524365412687682781",
                "4900829043004737418",
                "6195884386932410966",
                "13554217876979843574",
                "17902692039595931737",
                "12433028734895890975",
                "15971442058448435996",
                "4591894758077129763",
                "11258250015882429548",
                "16399550288873254981",
                "8246389845141771315",
                "14040203746442788850",
                "7283856864330834987",
                "12297563098718697441",
                "13560928146585163504",
                "7380926829734048483",
                "14591299561622291080",
                "8439722381984777599",
                "17375431987296514829",
                "16727607878674407272",
                "3233954801381564296",
                "17255435698225160983",
                "15093748890170255670",
                "15810389980847260072",
                "11120056430439037392",
                "5866130971823719482",
                "13327552690270163501",
            ],
        );

        inputs.insert(
            "base_message".to_string(),
            vec![
                "18114495772705111902",
                "2254271930739856077",
                "2068851770",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
                "0",
            ],
        );
        let inputs_bigint = inputs
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.iter()
                        .map(|s| BigInt::from_str(s).unwrap())
                        .collect::<Vec<BigInt>>(),
                )
            })
            .collect();

        use std::time::Instant;
        let start = Instant::now();
        let _ = create_rsa_main_witness(inputs_bigint);
        let end = Instant::now();
        println!("Time taken for rsa_main: {:?}", end.duration_since(start));
    }
}
