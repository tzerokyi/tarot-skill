use serde_json::{json, Value};
use std::io::{self, Read};
use std::process::ExitCode;
use std::str::FromStr;
use tarot_core::{
    analyze, cards, draw, find_card, spreads, Draw, DrawRequest, Spread, TimeFactor,
};

const HELP: &str = r#"tarot — native Rust tarot engine

USAGE:
  tarot draw --spread <single|three|diamond|moon|horseshoe|celtic> [options]
  tarot analyze --input <file.json|-> [--json]
  tarot analyze --spread <spread> [options]
  tarot deck list [--json]
  tarot deck show <0..77|Chinese card name> [--json]
  tarot spreads list [--json]
  tarot spreads show <spread> [--json]
  tarot doctor [--json]
  tarot --version

DRAW OPTIONS:
  --question <text>        Optional question (never sent to a network service)
  --seed <u64>             Reproducible ChaCha20 seed; default uses OS entropy
  --time-factor <value>    morning | afternoon | night; default uses local hour
  --json                   Explicit JSON mode (JSON is also the default)
  --json-only              Legacy compatibility alias

The old script form 'tarot --spread three' is also accepted.
All structured results go to stdout; errors go to stderr with nonzero exit codes.
"#;

#[derive(Debug)]
struct CliError {
    code: &'static str,
    message: String,
    exit_code: u8,
}

impl CliError {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_arguments",
            message: message.into(),
            exit_code: 2,
        }
    }

    fn input(message: impl Into<String>) -> Self {
        Self {
            code: "input_error",
            message: message.into(),
            exit_code: 1,
        }
    }
}

#[derive(Default)]
struct DrawFlags {
    spread: Option<Spread>,
    question: String,
    seed: Option<u64>,
    time_factor: Option<TimeFactor>,
    input: Option<String>,
}

fn parse_draw_flags(args: &[String], allow_input: bool) -> Result<DrawFlags, CliError> {
    let mut flags = DrawFlags::default();
    let mut cursor = 0;
    while cursor < args.len() {
        let argument = args[cursor].as_str();
        if matches!(argument, "--json" | "--json-only") {
            cursor += 1;
            continue;
        }
        let value = args
            .get(cursor + 1)
            .ok_or_else(|| CliError::invalid(format!("missing value for {argument}")))?;
        match argument {
            "--spread" => {
                if flags.spread.is_some() {
                    return Err(CliError::invalid("--spread supplied more than once"));
                }
                flags.spread = Some(Spread::from_str(value).map_err(CliError::invalid)?);
            }
            "--question" => flags.question = value.clone(),
            "--seed" => {
                flags.seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| CliError::invalid("--seed must be an unsigned 64-bit integer"))?,
                );
            }
            "--time-factor" => {
                flags.time_factor =
                    Some(TimeFactor::from_str(value).map_err(CliError::invalid)?);
            }
            "--input" if allow_input => flags.input = Some(value.clone()),
            _ => return Err(CliError::invalid(format!("unknown option: {argument}"))),
        }
        cursor += 2;
    }
    Ok(flags)
}

fn make_request(flags: DrawFlags) -> Result<DrawRequest, CliError> {
    let spread = flags
        .spread
        .ok_or_else(|| CliError::invalid("--spread is required"))?;
    Ok(DrawRequest {
        spread,
        question: flags.question,
        seed: flags.seed,
        time_factor: flags.time_factor,
    })
}

fn read_draw(path: &str) -> Result<Draw, CliError> {
    let contents = if path == "-" {
        let mut buffer = String::new();
        io::stdin()
            .take(1024 * 1024 + 1)
            .read_to_string(&mut buffer)
            .map_err(|err| CliError::input(format!("stdin: {err}")))?;
        buffer
    } else {
        let metadata = std::fs::metadata(path)
            .map_err(|err| CliError::input(format!("cannot stat input: {err}")))?;
        if metadata.len() > 1024 * 1024 {
            return Err(CliError::input("input JSON exceeds 1 MiB"));
        }
        std::fs::read_to_string(path)
            .map_err(|err| CliError::input(format!("cannot read input: {err}")))?
    };
    if contents.len() > 1024 * 1024 {
        return Err(CliError::input("input JSON exceeds 1 MiB"));
    }
    serde_json::from_str(&contents)
        .map_err(|err| CliError::input(format!("invalid draw JSON: {err}")))
}

fn positional_args(args: &[String]) -> Result<Vec<&str>, CliError> {
    let mut values = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--json" | "--json-only" => {}
            value if value.starts_with('-') => {
                return Err(CliError::invalid(format!("unknown option: {value}")));
            }
            value => values.push(value),
        }
    }
    Ok(values)
}

fn execute(args: &[String]) -> Result<Option<Value>, CliError> {
    if args.is_empty() || args.iter().any(|arg| matches!(arg.as_str(), "-h" | "--help")) {
        print!("{HELP}");
        return Ok(None);
    }
    if args[0] == "-V" || args[0] == "--version" {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(None);
    }

    let result = match args[0].as_str() {
        "draw" => {
            let request = make_request(parse_draw_flags(&args[1..], false)?)?;
            json!(draw(&request))
        }
        "analyze" => {
            let flags = parse_draw_flags(&args[1..], true)?;
            let drawn = if let Some(ref path) = flags.input {
                if flags.spread.is_some() || flags.seed.is_some() || flags.time_factor.is_some() {
                    return Err(CliError::invalid(
                        "--input cannot be combined with --spread, --seed or --time-factor",
                    ));
                }
                read_draw(path)?
            } else {
                draw(&make_request(flags)?)
            };
            let report = analyze(&drawn)
                .map_err(|err| CliError::input(err.to_string()))?;
            json!({"draw": drawn, "analysis": report})
        }
        "deck" => {
            let values = positional_args(&args[1..])?;
            match values.as_slice() {
                ["list"] => json!(cards()),
                ["show", name] => json!(find_card(name)
                    .ok_or_else(|| CliError::invalid(format!("unknown card: {name}")))?),
                _ => return Err(CliError::invalid("use 'deck list' or 'deck show <id|name>'")),
            }
        }
        "spreads" => {
            let values = positional_args(&args[1..])?;
            match values.as_slice() {
                ["list"] => json!(spreads()),
                ["show", key] => {
                    let spread = Spread::from_str(key).map_err(CliError::invalid)?;
                    json!(spreads()
                        .into_iter()
                        .find(|item| item.key == spread)
                        .expect("all spreads exist in the catalog"))
                }
                _ => return Err(CliError::invalid("use 'spreads list' or 'spreads show <key>'")),
            }
        }
        "doctor" => {
            if !positional_args(&args[1..])?.is_empty() {
                return Err(CliError::invalid("doctor takes no positional arguments"));
            }
            json!({
                "status": "ok",
                "name": "tarot",
                "version": env!("CARGO_PKG_VERSION"),
                "runtime": "native-rust",
                "deck_size": tarot_core::CARD_COUNT,
                "spread_count": Spread::ALL.len(),
                "seeded_rng": "chacha20",
                "mcp_transport": "not_implemented"
            })
        }
        flag if flag.starts_with('-') => {
            let request = make_request(parse_draw_flags(args, false)?)?;
            json!(draw(&request))
        }
        value => return Err(CliError::invalid(format!("unknown command: {value}"))),
    };
    Ok(Some(result))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match execute(&args) {
        Ok(Some(value)) => match serde_json::to_string_pretty(&value) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("{}", json!({"error":{"code":"serialization_error","message":err.to_string()}}));
                ExitCode::from(1)
            }
        },
        Ok(None) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", json!({"error":{"code":err.code,"message":err.message}}));
            ExitCode::from(err.exit_code)
        }
    }
}
