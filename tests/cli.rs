use serde_json::Value;
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tarot"))
        .args(args)
        .output()
        .expect("tarot binary should be built by cargo test")
}

fn json(args: &[&str]) -> Value {
    let result = run(args);
    assert!(result.status.success(), "stderr: {}", String::from_utf8_lossy(&result.stderr));
    serde_json::from_slice(&result.stdout).expect("stdout should be JSON")
}

#[test]
fn seeded_draw_and_json_analysis() {
    let args = ["draw", "--spread", "three", "--seed", "123", "--time-factor", "night"];
    let first = json(&args);
    let second = json(&args);
    assert_eq!(first, second);
    assert_eq!(first["seed"], 123);
    assert_eq!(first["time_factor"], "night");
    assert_eq!(first["cards"].as_array().unwrap().len(), 3);
    assert_eq!(json(&["analyze", "--spread", "three", "--seed", "123", "--time-factor", "night"])["analysis"]["relations"].as_array().unwrap().len(), 2);
}

#[test]
fn legacy_arguments_and_catalog_are_accessible() {
    let first = json(&["draw", "--spread", "single", "--seed", "7", "--time-factor", "morning"]);
    let legacy = json(&["--spread", "single", "--seed", "7", "--time-factor", "morning"]);
    assert_eq!(first, legacy);
    assert_eq!(json(&["deck", "list"]).as_array().unwrap().len(), 78);
    assert_eq!(json(&["deck", "show", "0"])["name"], "愚者");
    assert_eq!(json(&["spreads", "list"]).as_array().unwrap().len(), 6);
    assert_eq!(json(&["doctor"])["status"], "ok");
}

#[test]
fn invalid_arguments_use_stderr_and_failure_codes() {
    let result = run(&["draw", "--spread", "unknown"]);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let error: Value = serde_json::from_slice(&result.stderr).unwrap();
    assert_eq!(error["error"]["code"], "invalid_arguments");
}
