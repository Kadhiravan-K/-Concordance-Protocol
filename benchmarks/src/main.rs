use std::env;

#[derive(Debug, Clone, Copy)]
struct IntegrationModel {
    schemes: usize,
    adapter_implementations: usize,
    bilateral_implementations: usize,
    adapter_conformance_suites: usize,
    bilateral_conformance_suites: usize,
}

impl IntegrationModel {
    fn for_schemes(schemes: usize) -> Self {
        let pairs = schemes * schemes.saturating_sub(1) / 2;
        Self {
            schemes,
            adapter_implementations: schemes,
            bilateral_implementations: pairs,
            adapter_conformance_suites: schemes,
            bilateral_conformance_suites: pairs,
        }
    }

    fn csv_header() -> &'static str {
        "schemes,adapter_implementations,bilateral_implementations,adapter_conformance_suites,bilateral_conformance_suites"
    }

    fn csv_row(&self) -> String {
        format!(
            "{},{},{},{},{}",
            self.schemes,
            self.adapter_implementations,
            self.bilateral_implementations,
            self.adapter_conformance_suites,
            self.bilateral_conformance_suites
        )
    }
}

fn main() {
    let format = env::args().skip(1).collect::<Vec<_>>().windows(2).find(|window| window[0] == "--format").map(|window| window[1].clone()).unwrap_or_else(|| "csv".into());
    if format != "csv" && format != "text" {
        eprintln!("Usage: concordance-integration-benchmark [--format csv|text]");
        std::process::exit(2);
    }
    let models: Vec<_> = [1usize, 2, 4, 8, 16].into_iter().map(IntegrationModel::for_schemes).collect();
    if format == "csv" {
        println!("{}", IntegrationModel::csv_header());
        for model in models { println!("{}", model.csv_row()); }
    } else {
        println!("Phase 2 integration-count model");
        for model in models {
            println!("schemes={}: adapters={} bilateral_pairs={} adapter_suites={} bilateral_suites={}", model.schemes, model.adapter_implementations, model.bilateral_implementations, model.adapter_conformance_suites, model.bilateral_conformance_suites);
        }
        println!("This proves only count growth. Replace model columns with measured LOC, implementation hours, and conformance hours before claiming effort growth.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_counts_have_linear_and_quadratic_shapes() {
        let sixteen = IntegrationModel::for_schemes(16);
        assert_eq!(sixteen.adapter_implementations, 16);
        assert_eq!(sixteen.bilateral_implementations, 120);
    }
}
