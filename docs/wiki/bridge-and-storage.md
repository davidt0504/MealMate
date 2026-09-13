# Bridge and storage

## Responsibility

This subsystem is the only route from Flutter to the Rust core and the only implementation that touches SQLite. Hand-written bridge modules map stable DTOs and typed failures onto domain/application/storage calls. Generated Dart and Rust files provide serialization, dispatch, and native loading and must be regenerated rather than edited.

## Bridge API

`rust/src/api/mod.rs` publishes decisions, error, health, household, pantry, planned-meals, planner, planning-cycle, recipe, restrictions, shopping, and starter modules. Their Dart counterparts under `lib/src/rust/api` are generated service functions and DTO classes.

The main commands are task-sized: open/export/restore/reset database; bootstrap/rename/onboard household; save/list/archive recipes; load/save restrictions and pantry marks; manage planning cycles and planned meals; derive and mutate shopping views; preview/apply a covered cycle; record a user decision; and install starter content. UUID creation stays in the bridge, keeping pure domain/application logic deterministic and injectable in tests.

`KimattaError` is the public failure taxonomy. Conversion layers map validation, ownership, planning, storage, and application errors to user-safe categories. The bridge parses string IDs and civil dates before calling deeper layers, and converts domain objects back into complete DTOs.

## Connection owner

`rust/src/db.rs` owns a process-wide `Mutex<Option<Connection>>`. Database open establishes and stores the connection; storage-touching bridge calls use the same serialized connection. Restore/reset replace database state through the health API and downstream Dart invalidation. A poisoned or unopened connection becomes a typed bridge failure rather than a handle crossing FFI.

This serialization makes concurrent FRB futures safe at the connection boundary and ensures migrations/bootstrap are not independently reopened per feature call.

## SQLite adapter

`kimatta-storage` owns connection configuration, migrations, backup validation, restore, and household-scoped repositories. The schema has evolved through version 11. Major tables cover households and members, planning cycles and enabled slots, global/catalog and custom ingredients, recipes/provenance/lines, restrictions and preferences, planned meals/components, pantry marks, shopping line state/manual items, policies, and the append-only controller ledger.

Writes use transactions, commonly `IMMEDIATE` where a read-modify-write sequence must exclude races. Public transaction-scoped helpers allow the application service to compose mutation plus evidence under one commit. Ownership probes ensure a foreign household's entity is absent/refused rather than reassigned. Automation cannot overwrite or delete user-locked meals.

Recipe persistence retains original ingredient text and explicit provenance/rights fields. Archive markers preserve referential history. Starter installation checks held catalog IDs and per-household slugs, seeds missing content in one transaction, and is idempotent without resurrecting an archived starter recipe.

Shopping derivation reads planned occurrences and referenced recipes, while separately stored UI state is keyed by household and exact cycle window. Quantity tokens detect when a previously checked derived line has changed. Reset removes window state and checked manual items while retaining unchecked wants.

## Controller persistence

`controller.rs` loads a planning snapshot spanning cycle configuration, plans, recipes, restrictions, preferences, pantry, and policies. It persists policies, applies generated plans, and appends immutable ledger entries. Apply-plus-record shares one transaction so state and its audit evidence cannot diverge. Ledger sequence and triggers preserve historical ordering and immutability.

## Backup and recovery

Export uses SQLite's consistent-copy mechanisms and reports schema metadata. Restore validates that an input is a readable Kimatta database with a supported schema before replacing live state. Reset recreates a clean database. Dart performs the platform file selection/sharing and then invalidates all cached providers after a successful swap.

## Coverage evidence

This page owns 38 files: 19 generated Dart bridge files, the hand-written Rust bridge/API and generated Rust runtime, plus the storage crate. Large generated files are covered as generated artifacts; the two first-party storage source files receive behavioral coverage here. Exact classification and hashes are in `../coverage-manifest.json`.

