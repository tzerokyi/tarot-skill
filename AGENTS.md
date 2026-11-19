# Tarot Skill implementation contract

- Native Rust only. Public business logic belongs to `src/lib.rs`, `src/catalog.rs`, and `src/analysis.rs`. CLI (`src/main.rs`) is a thin adapter; future MCP transport must call the same library, not duplicate algorithms.
- This repository owns the tarot source semantics. Preserve 78-card ordering, Chinese names, six spread positions, per-position major weighting, orientation probabilities, time-factor boosts, nonreplacement draws, and the JSON field names from the original Python implementation. Explain intentional RNG/seed changes in `docs/RUST_MIGRATION.md`.
- The original reference documents in `references/` and `SKILL.md` remain the interpretation/safety source, not automatically generated predictions. Do not turn guidance into deterministic personal, medical, legal or financial pronouncements.
- Stage Rust changes on `feat/` or `fix/` branches, format using the exact installed rustfmt, and verify fmt, strict Clippy, tests, binary build, JSON smoke before any main promotion.
- Use the trusted self-hosted Linux GitHub runner (12 build jobs where supported). Do not run untrusted fork PRs on it. Keep Cargo caches warm. No built binaries, `target/`, Python runtime shims, GUI/JS/Tauri/Electron or release artifacts in Git.
- Keep cross-platform paths and inputs portable; plan a Windows compile gate before calling Windows verified.
- Crate publication is a later decision: avoid premature network server/MCP dependency, keep public API explicit and documented, and reserve semver changes for breaking contracts.
