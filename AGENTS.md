# Agent instructions - gotoo_vid2txt

1. Inspect Git HEAD, status, tree, relevant files, Cargo.lock, tests and CI before editing.
2. Stop if there are unexpected changes in the working tree. Never overwrite user work.
3. Do not claim remote repository code exists if the GitHub repository is empty.
4. Scope changes to the requested feature or bug; avoid speculative workspaces, plugin systems and alternate engines not supported by tests.
5. Preserve separation: domain / config / registry / acquisition / audio / transcription / export / output / pipeline / CLI.
6. Every failure fix must include a regression test. Never weaken CI, clippy or tests to make a change pass.
7. Keep local files and outputs private; never log private transcript content, tokens or auth URLs.
8. Always run: cargo fmt --all -- --check; cargo test --all-targets; cargo clippy --all-targets --all-features -- -D warnings; RUSTDOCFLAGS='-D warnings' cargo doc --no-deps.
9. Execute real FFmpeg / Whisper / yt-dlp integration only when dependencies and legal test media are available. Report blockers, do not invent PASS.
10. Final report: commit SHA and diff summary, exact validations/results, untested areas, security concerns, blockers; stop before push or release unless explicitly instructed.

11. Reject unknown backend IDs, configuration typos and silent fallback. Explicitly document which integrations are implemented rather than planned.
12. A new backend must have selection tests, a positive contract test, an injected failure test, and at least one real-tool smoke test before any production-ready claim.
