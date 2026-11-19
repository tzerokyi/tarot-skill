# 🔮 tarot-skill — Native Rust tarot core

기존 Python `scripts/draw.py`를 **순수 네이티브 Rust 라이브러리 + CLI**로 이식한 프로젝트. UI/웹/서버/파이썬 런타임 없음. 이후 crates.io 배포 또는 MCP 어댑터 추가가 쉽도록 API를 분리했다.

中文：原来的塔罗抽牌脚本已迁移为 Rust 核心库 `tarot_core` + 命令行程序 `tarot`。解读规则仍保存在 `SKILL.md` 和 `references/`。

## Quick start

```sh
cargo run -- draw --spread single --seed 42 --time-factor morning
cargo run -- draw --spread three --question "事业方向" --seed 42 --time-factor night --json
cargo run -- analyze --spread celtic --seed 42 --time-factor night
cargo run -- deck list --json
cargo run -- deck show 0
cargo run -- spreads list
cargo run -- doctor --json
cargo test
```

Install a local native CLI: `cargo install --path .`. Then use `tarot draw ...`. Default output is valid JSON; errors are JSON on stderr, with a nonzero exit code.

For scripted agents, pass **both** `--seed` and `--time-factor` to make a reading reproducible. Otherwise the seed is sampled from OS entropy and the time factor follows the local hour. The deterministic RNG is ChaCha20; the same numeric seed **does not** reproduce Python's Mersenne Twister card sequence.

### CLI

| Command | Description |
| --- | --- |
| `draw --spread <id>` | Weighted, no-replacement draw with upright/reversed orientations |
| `analyze --spread <id>` | Draw and report element/suit/major ratios and adjacent card relations |
| `analyze --input reading.json` | Validate/analyze an existing JSON draw (`-` for stdin) |
| `deck list / show <id|name>` | Inspect all 78 cards or one card |
| `spreads list / show <id>` | Inspect the 6 spreads and ordered positions |
| `doctor` | Headless runtime/capability diagnostic |

Spread IDs: `single`, `three`, `diamond`, `moon`, `horseshoe`, `celtic`.

### Rust library

```rust
use tarot_core::{analyze, draw, DrawRequest, Spread, TimeFactor};

let request = DrawRequest::new(Spread::Three)
    .with_question("事业")
    .with_seed(42)
    .with_time_factor(TimeFactor::Night);
let reading = draw(&request);
let analysis = analyze(&reading)?;
# Ok::<(), tarot_core::TarotError>(())
```

All domain operations are exposed independently of the CLI. A future optional MCP/stdin adapter should consume these functions rather than implement a second tarot engine.

## Design and migration notes

- `src/catalog.rs`: 78-card canonical catalog, suits/elements, 6 spread definitions.
- `src/lib.rs`: typed requests/results, secure seeded draws, external JSON validation.
- `src/analysis.rs`: adjacent relations from A1/A2/A3/B1/B2/B3 plus known classic pairs, suit and major ratios.
- `src/main.rs`: thin CLI and machine-readable errors.
- `tests/`: binary-level smoke and invalid-input checks.
- `SKILL.md`, `references/`: interpretation/safety workflow. The engine returns **structured signals, not certain predictions**.
- `docs/RUST_MIGRATION.md`: exact parity matrix and intentionally different RNG behavior.

This repository declares MIT in its original README; see `LICENSE`. No build products, caches or native executables are committed. Rust checks/builds run through the trusted self-hosted CI workflow on the staging branch. A Windows target has not yet been verified.
