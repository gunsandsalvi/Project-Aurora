//! The probe as a command. Everything measurable is in the library; this chooses a budget and prints.
//!
//! `--device` takes §12's own figures — a fifteen-minute soak and an 8 GiB allocation walk. Without
//! it the run is a host floor and a smoke test, because a fifteen-minute CI job measuring a desktop's
//! thermal behaviour tells nobody anything.

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let device = args.iter().any(|a| a == "--device");
    let budget = if device {
        aurora_probe::Budget::device()
    } else {
        aurora_probe::Budget::host()
    };
    let scratch = std::env::temp_dir();
    let commit = option_env!("AURORA_COMMIT").unwrap_or("unknown");
    let label = args
        .iter()
        .position(|a| a == "--device-name")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| {
            if device {
                "android".to_owned()
            } else {
                "host".to_owned()
            }
        });

    let (m, sink) = aurora_probe::measure(&budget, &scratch);
    println!("{}", aurora_probe::json(&m, commit, &label, sink));
}
