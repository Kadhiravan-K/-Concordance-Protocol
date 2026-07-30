use std::env;

use concordance_simulator::{run, ScenarioConfig, SimulationResult};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|argument| argument == "--help" || argument == "-h") {
        print_help();
        return;
    }
    let config = match parse_config(&args) {
        Ok(config) => config,
        Err(error) => { eprintln!("error: {error}\n"); print_help(); std::process::exit(2); }
    };
    let format = option(&args, "--format").unwrap_or_else(|| "text".into());
    match run(config) {
        Ok(result) if format == "csv" => { println!("{}", SimulationResult::csv_header()); println!("{}", result.to_csv_row()); }
        Ok(result) if format == "text" => print_result(&config, &result),
        Ok(_) => { eprintln!("error: format must be text or csv"); std::process::exit(2); }
        Err(error) => { eprintln!("error: {error}"); std::process::exit(1); }
    }
}

fn parse_config(args: &[String]) -> Result<ScenarioConfig, String> {
    let defaults = ScenarioConfig::default();
    Ok(ScenarioConfig {
        agents: number(args, "--agents", defaults.agents)?,
        max_schemes_per_agent: number(args, "--max-schemes", defaults.max_schemes_per_agent)?,
        adversarial_percent: number(args, "--adversarial-percent", defaults.adversarial_percent)?,
        revoked_percent: number(args, "--revoked-percent", defaults.revoked_percent)?,
        expired_percent: number(args, "--expired-percent", defaults.expired_percent)?,
        conflict_percent: number(args, "--conflict-percent", defaults.conflict_percent)?,
        seed: number(args, "--seed", defaults.seed)?,
    })
}

fn option(args: &[String], name: &str) -> Option<String> { args.windows(2).find(|window| window[0] == name).map(|window| window[1].clone()) }
fn number<T: std::str::FromStr>(args: &[String], name: &str, default: T) -> Result<T, String> { option(args, name).map(|value| value.parse().map_err(|_| format!("{name} requires a number"))).unwrap_or(Ok(default)) }

fn print_result(config: &ScenarioConfig, result: &SimulationResult) {
    println!("Phase 2 deterministic simulation");
    println!("agents={} max_schemes={} seed={}", config.agents, config.max_schemes_per_agent, config.seed);
    println!("adversarial={} revoked={} expired={} conflicts={}", result.adversarial_agents, result.revoked_agents, result.expired_agents, result.conflict_agents);
    println!("allows: naive={} correlation-capped={}", result.naive_allows, result.capped_allows);
    println!("adversarial allows: naive={} correlation-capped={}", result.naive_adversarial_allows, result.capped_adversarial_allows);
    println!("capped conflicts={} envelopes={} estimated_state_bytes={} elapsed_micros={}", result.capped_conflicts, result.envelopes, result.estimated_state_bytes, result.elapsed_micros);
    println!("Interpretation: a lower capped adversarial-allow count is evidence for declared-source capping in this synthetic scenario only.");
}

fn print_help() {
    println!("Usage: concordance-simulate [--agents 10..1000] [--max-schemes 1..3] [--adversarial-percent 0..100] [--revoked-percent 0..100] [--expired-percent 0..100] [--conflict-percent 0..100] [--seed N] [--format text|csv]");
}
