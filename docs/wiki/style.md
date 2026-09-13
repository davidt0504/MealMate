# Architecture wiki style guide

## Purpose

Keep this wiki useful as a map of implemented behavior, not a second product roadmap. Source code and committed configuration are architecture evidence. Plans, task cards, and uncommitted work are context and must be labelled as such.

## Page rules

- Keep one subsystem concern per page and each page below 300 lines. A coverage-heavy page may reach 400 lines only when splitting would obscure ownership.
- Begin with the subsystem's responsibility, then describe entry points, state ownership, behavior, failure boundaries, and tests.
- Give load-bearing symbols behavioral prose: what initiates them, what they read or mutate, what they return, and which invariant they protect.
- Prefer stable symbol names and paths over line numbers. Line numbers age quickly.
- Distinguish hand-written source from generated output. Never present generated FRB implementation details as a manual extension point.
- Do not claim a future card, optional adapter, or documented aspiration is implemented.

## Coverage rules

- The index is the sole human-facing path-to-page routing map.
- `coverage-manifest.json` is the exact machine-readable assignment and fingerprint source.
- Every tracked first-party file receives exactly one page assignment. Record Python definitions when parsing succeeds and record parse failures explicitly.
- Mark symlinks, submodules, and binaries distinctly. Roll up only vendored or asset trees in prose; never hide first-party source behind a directory count.
- Run the complement check after every inventory change. Zero uncovered and zero duplicate paths is required.

## Incremental update rules

- Architecture pages advance only from committed changes since `state.json:last_run.commit`.
- Describe uncommitted changes in the change brief without folding them into the architecture.
- Update only pages reached by committed changed paths. If more than four pages are affected, retain them as pending until cost is approved.
- Set `last_run.timestamp` to the run-start timestamp so commits made during a run cannot be skipped.
- Rebuild the embedded viewer and preserve brief filenames; chronology is filename-based.

## Voice

Use direct, plain prose. Explain boundaries and consequences. Avoid promotional language, speculation, and unexplained acronyms. Briefs must be listenable: ASCII punctuation, short paragraphs, explicit spoken section signposts, and no tables or code blocks.

