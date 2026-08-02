use std::{fs, path::PathBuf};

use clap::Parser;
use concordance_adapters::{ConformanceCoverage, FixtureSourceClass};
use serde_json::Value;

use concordance_certification::{run_certification_suite, CertificationResult};

#[derive(Debug, Parser)]
#[command(name = "concordance-certify", about = "Run the Concordance Certification Suite.")]
struct Args {
    /// Directory containing published adapter conformance reports.
    #[arg(long)]
    reports_dir: PathBuf,
}

fn main() {
    let args = Args::parse();

    let result = run_certification_suite(&args.reports_dir).unwrap_or_else(|err| {
        eprintln!("Certification suite failed: {err}");
        std::process::exit(1);
    });

    if !result.passed {
        eprintln!("Certification suite did not pass.");
        for failure in result.failures {
            eprintln!("- {failure}");
        }
        std::process::exit(1);
    }

    println!("Certification suite passed.");
}
