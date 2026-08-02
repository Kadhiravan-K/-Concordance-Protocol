use std::{collections::HashMap, env, fs, io::{self, Write}};

use concordance_core::TrustObjectEnvelope;

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let path = args.next().unwrap_or_default();

    match command.as_str() {
        "inspect" => run_inspect(&path),
        "verify" => run_verify(&path),
        "summary" => run_summary(&path),
        "interactive" => run_interactive(&path),
        "help" | "--help" | "-h" | "" => print_usage(),
        _ => {
            eprintln!("Unknown command: {command}\n");
            print_usage();
            std::process::exit(2);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: concordance <command> <bundle.json>");
    eprintln!("Commands:");
    eprintln!("  inspect      Print each envelope summary from a JSON bundle");
    eprintln!("  verify       Validate signatures and bindings for each envelope");
    eprintln!("  summary      Report high-level counts and validity metrics");
    eprintln!("  interactive  Open a simple interactive bundle inspector");
}

fn load_bundle(path: &str) -> Vec<TrustObjectEnvelope> {
    let contents = fs::read_to_string(path).expect("bundle must be readable");
    serde_json::from_str(&contents).expect("bundle must be an array of TOE JSON objects")
}

fn run_inspect(path: &str) {
    if path.is_empty() {
        print_usage();
        std::process::exit(2);
    }
    let bundle = load_bundle(path);
    for envelope in bundle {
        let verification = envelope.verify().map(|_| "valid").unwrap_or("INVALID");
        println!("{} {} subject={} strength={:.3} {verification}", envelope.envelope_id, envelope.claim_class, envelope.subject, envelope.normalized_strength);
    }
}

fn run_verify(path: &str) {
    if path.is_empty() {
        print_usage();
        std::process::exit(2);
    }
    let bundle = load_bundle(path);
    let mut failures = 0;
    for envelope in bundle {
        match envelope.verify() {
            Ok(_) => println!("OK {} {}", envelope.envelope_id, envelope.claim_class),
            Err(err) => {
                failures += 1;
                println!("FAIL {} {}: {err}", envelope.envelope_id, envelope.claim_class);
            }
        }
    }
    if failures > 0 {
        std::process::exit(1);
    }
}

fn run_summary(path: &str) {
    if path.is_empty() {
        print_usage();
        std::process::exit(2);
    }
    let bundle = load_bundle(path);
    let total = bundle.len();
    let mut claim_counts = HashMap::new();
    let mut issuer_counts = HashMap::new();
    let mut valid = 0;

    for envelope in bundle {
        *claim_counts.entry(envelope.claim_class.clone()).or_insert(0) += 1;
        *issuer_counts.entry(envelope.issuer.clone()).or_insert(0) += 1;
        if envelope.verify().is_ok() {
            valid += 1;
        }
    }

    println!("Total envelopes: {total}");
    println!("Valid envelopes: {valid}");
    println!("Invalid envelopes: {}", total.saturating_sub(valid));
    println!("\nClaim class counts:");
    for (claim, count) in claim_counts {
        println!("  {claim}: {count}");
    }
    println!("\nIssuer counts:");
    for (issuer, count) in issuer_counts {
        println!("  {issuer}: {count}");
    }
}

fn run_interactive(path: &str) {
    if path.is_empty() {
        print_usage();
        std::process::exit(2);
    }
    let bundle = load_bundle(path);
    if bundle.is_empty() {
        println!("Bundle is empty.");
        return;
    }

    println!("Loaded {} envelopes. Type a number to inspect details, 'q' to quit.", bundle.len());
    for (idx, envelope) in bundle.iter().enumerate() {
        let status = if envelope.verify().is_ok() { "valid" } else { "INVALID" };
        println!("[{idx}] {} {} subject={} strength={:.3} {status}", envelope.envelope_id, envelope.claim_class, envelope.subject, envelope.normalized_strength);
    }

    let stdin = io::stdin();
    loop {
        print!("Choose envelope> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }
        let choice = line.trim();
        if choice.eq_ignore_ascii_case("q") || choice.eq_ignore_ascii_case("quit") {
            break;
        }
        if let Ok(idx) = choice.parse::<usize>() {
            if idx < bundle.len() {
                let envelope = &bundle[idx];
                println!("\nEnvelope {}:\n{}\n", idx, serde_json::to_string_pretty(envelope).unwrap());
                continue;
            }
        }
        println!("Enter a number between 0 and {} or q to quit.", bundle.len().saturating_sub(1));
    }
}
