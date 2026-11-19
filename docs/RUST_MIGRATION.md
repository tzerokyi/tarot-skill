# Python → native Rust contract

Source frozen from `scripts/draw.py` at upstream fork commit `a9f59f02b3df8a539e9ec618fb4757d481443803`.

| Feature | Rust status | Notes |
| --- | --- | --- |
| 22 majors + 56 minors, Chinese names/ordering | Ported | Stable numeric ids 0–77 |
| Six spreads + exact position order | Ported | `Spread` enum and `spreads()` |
| Draw without replacement | Ported | One selected card removed per position |
| Position major bias | Ported | 60/28 for marked positions, otherwise 1 |
| Time-factor element boost | Ported | 1.08x; 06–11 morning, 12–17 afternoon, otherwise night |
| Upright probability | Ported | 0.7 for marked positions, 0.6 otherwise |
| No explicit seed | Ported | OS CSPRNG provides u64 seed; returned for reproduction |
| Explicit seed | Intentional change | ChaCha20 deterministic Rust stream, **not** Python MT19937; same seed does not reproduce old Python draw |
| Python negative/huge seeds | Intentional change | Rust CLI accepts only u64 for a portable, typed contract |
| JSON output | Ported | Exact original field names/Chinese orientation text |
| Interpretation method | Preserved as reference | `SKILL.md` and `references/` remain human/agent guidance |
| Adjacent relation heuristics | New structured API | A1/A2/A3/B1/B2/B3 and classic pairs; hints, not predictions |
| MCP server | Deferred | Stable Rust functions + JSON CLI form the future adapter boundary |
| GUI | Not in scope | CLI/core-only project |

## Public crate boundary

The package currently exposes a library called `tarot_core` and a binary called `tarot`. Key functions: `cards`, `card_info`, `find_card`, `spreads`, `draw`, `validate_draw`, `analyze`, `relate`.

Calling `draw` requires a typed `DrawRequest`. Supply **both** `seed` and `time_factor` to reproduce a draw, as local time is otherwise a variable. JSON validation rejects invented cards, duplicate cards, altered spread positions and inconsistent major flags.

This is an offline library. No UI, daemon, external Python, interpreter, network, or mandatory MCP transport.
