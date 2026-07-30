fn main() {
    println!("n,adapter_integrations,bilateral_integrations,adapter_model_cost,bilateral_model_cost");
    for n in [1usize, 2, 4, 8, 16] {
        // These are transparent work-unit models. Replace with measured LOC and
        // engineer-hours after independently implementing each fixture suite.
        let adapters = n;
        let bilateral = n * n.saturating_sub(1) / 2;
        println!("{n},{adapters},{bilateral},{adapters},{bilateral}");
    }
    println!("NOTE: this benchmark defines the falsifiable integration-count baseline; it does not prove adapter effort is constant.");
}
