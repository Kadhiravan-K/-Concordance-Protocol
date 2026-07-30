use std::{env, fs};

use concordance_core::TrustObjectEnvelope;

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let path = args.next().unwrap_or_default();
    if command != "inspect" || path.is_empty() {
        eprintln!("Usage: concordance inspect <bundle.json>");
        std::process::exit(2);
    }
    let contents = fs::read_to_string(path).expect("bundle must be readable");
    let bundle: Vec<TrustObjectEnvelope> = serde_json::from_str(&contents).expect("bundle must be an array of TOE JSON objects");
    for envelope in bundle {
        let verification = envelope.verify().map(|_| "valid").unwrap_or("INVALID");
        println!("{} {} subject={} strength={:.3} {verification}", envelope.envelope_id, envelope.claim_class, envelope.subject, envelope.normalized_strength);
    }
}
