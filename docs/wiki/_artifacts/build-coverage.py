#!/usr/bin/env python3
"""Build the tracked-file coverage evidence for the Meal Mate code report."""

from __future__ import annotations

import ast
import hashlib
import json
import re
import subprocess
from collections import defaultdict
from pathlib import Path

REPO = Path("/home/davidlinux/projects/personal/meal_mate")
HERE = Path(__file__).resolve().parent

PAGES = {
    "system-overview.md": "System shape, package manifests, and repository-level configuration",
    "flutter-shell.md": "Flutter entry point, app shell, navigation, and native build hook",
    "flutter-features.md": "Feature screens, Riverpod state, presentation helpers, and workflows",
    "bridge-and-storage.md": "Generated FRB bindings, hand-written bridge API, connection owner, and SQLite adapter",
    "rust-domain-and-planner.md": "Household kernel, food model, controller application service, and planner",
    "tests-platform-and-tools.md": "Tests, Android host/packaging, automation, hooks, and developer tools",
    "product-and-governance.md": "Product requirements, roadmap, task system, research, reviews, and reference data",
}


def page_for(path: str) -> str:
    if path.startswith("lib/features/"):
        return "flutter-features.md"
    if path in {"lib/main.dart"} or path.startswith("lib/app/") or path.startswith("hook/"):
        return "flutter-shell.md"
    if path.startswith("lib/src/rust/") or path.startswith("rust/src/") or path.startswith("rust/crates/kimatta-storage/"):
        return "bridge-and-storage.md"
    if path.startswith("rust/crates/household-core/") or path.startswith("rust/crates/food-domain/") or path.startswith("rust/crates/kimatta-application/"):
        return "rust-domain-and-planner.md"
    if path.startswith(("test/", "android/", "tools/", ".github/", ".githooks/")):
        return "tests-platform-and-tools.md"
    if path.startswith(("docs/", "refs/")) or path.startswith("KNOWN_ISSUES") or path == ".impeccable.md":
        return "product-and-governance.md"
    return "system-overview.md"


def declarations(path: str, data: bytes) -> tuple[list[str], str | None]:
    if b"\0" in data:
        return [], None
    text = data.decode("utf-8", errors="replace")
    if path.endswith(".py"):
        try:
            tree = ast.parse(text, filename=path)
        except SyntaxError as exc:
            return [], f"SyntaxError at line {exc.lineno}: {exc.msg}"
        return [
            f"{type(node).__name__}:{node.name}"
            for node in tree.body
            if isinstance(node, (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef))
        ], None
    if path.endswith(".dart"):
        found = re.findall(r"(?m)^(?:abstract |base |final |sealed )?(?:class|enum|mixin|extension|typedef)\s+([A-Za-z_]\w*)", text)
        return found, None
    if path.endswith(".rs"):
        found = re.findall(r"(?m)^pub(?:\([^)]*\))?\s+(?:async\s+)?(?:fn|struct|enum|trait|type|const|static|mod)\s+([A-Za-z_]\w*)", text)
        return found, None
    return [], None


def main() -> None:
    rows = subprocess.check_output(
        [
            "git",
            "ls-files",
            "-s",
            "-z",
            "--",
            ".",
            ":(exclude)docs/wiki/**",
            ":(exclude)docs/briefs/**",
        ],
        cwd=REPO,
    ).split(b"\0")
    pages: dict[str, list[dict[str, object]]] = defaultdict(list)
    all_paths: list[str] = []
    parse_failures: list[dict[str, str]] = []
    counts = defaultdict(int)
    for raw in rows:
        if not raw:
            continue
        meta, raw_path = raw.split(b"\t", 1)
        mode, object_id, stage = meta.decode().split()
        path = raw_path.decode()
        full = REPO / path
        if mode == "160000":
            kind, data = "submodule", b""
        elif mode == "120000":
            kind, data = "symlink", full.readlink().as_posix().encode()
        else:
            data = full.read_bytes()
            kind = "binary" if b"\0" in data else "text"
        symbols, failure = declarations(path, data)
        if failure:
            parse_failures.append({"path": path, "error": failure})
        record = {
            "path": path,
            "git_mode": mode,
            "git_object": object_id,
            "stage": int(stage),
            "kind": kind,
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
            "symbols": symbols,
            "parse_failure": failure,
        }
        pages[page_for(path)].append(record)
        all_paths.append(path)
        counts[kind] += 1

    mapped = {item["path"] for records in pages.values() for item in records}
    complement = sorted(set(all_paths) - mapped)
    duplicates = sorted(path for path in mapped if sum(path == item["path"] for records in pages.values() for item in records) != 1)
    payload = {
        "version": 1,
        "repository": str(REPO),
        "tracked_file_count": len(all_paths),
        "classification_counts": dict(sorted(counts.items())),
        "python_parse_failures": parse_failures,
        "pages": {
            page: {"purpose": PAGES[page], "files": sorted(pages[page], key=lambda item: item["path"])}
            for page in PAGES
        },
        "complement_check": {
            "uncovered": complement,
            "duplicates": duplicates,
            "passed": not complement and not duplicates and len(all_paths) == sum(len(v) for v in pages.values()),
        },
    }
    (HERE / "coverage-manifest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        "tracked": len(all_paths),
        "pages": {page: len(pages[page]) for page in PAGES},
        "classifications": dict(counts),
        "python_parse_failures": len(parse_failures),
        "uncovered": len(complement),
        "duplicates": len(duplicates),
        "passed": payload["complement_check"]["passed"],
    }, indent=2))


if __name__ == "__main__":
    main()
