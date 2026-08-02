use std::{fs, path::PathBuf};

use concordance_core::TrustObjectEnvelope;

fn main() {
    let example_path = PathBuf::from("bundle.json");
    if !example_path.exists() {
        eprintln!("Expected examples/standalone/bundle.json to exist. Create a JSON bundle of TrustObjectEnvelope objects first.");
        std::process::exit(1);
    }

    let contents = fs::read_to_string(&example_path).expect("bundle must be readable");
    let bundle: Vec<TrustObjectEnvelope> = serde_json::from_str(&contents).expect("bundle must be an array of TOE JSON objects");

    println!("Loaded {} envelopes", bundle.len());
    for (idx, envelope) in bundle.iter().enumerate() {
        let verification = envelope.verify().map(|_| "valid").unwrap_or("INVALID");
        println!("[{}] {} {} subject={} strength={:.3} {verification}", idx, envelope.envelope_id, envelope.claim_class, envelope.subject, envelope.normalized_strength);
    }

    if let Some(first) = bundle.first() {
        println!("\nFirst envelope full JSON:");
        println!("{}", serde_json::to_string_pretty(first).unwrap());
    }
}
