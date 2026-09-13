use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest directory"))
        .join("../../../docs/research/MVP-032_STARTER_PROVENANCE_MANIFEST.json");
    println!("cargo:rerun-if-changed={}", manifest.display());

    let source = fs::read_to_string(&manifest).expect("read starter provenance manifest");
    let mut content: serde_json::Value =
        serde_json::from_str(&source).expect("parse starter provenance manifest");
    let policy = content
        .get("policy")
        .and_then(serde_json::Value::as_object)
        .expect("starter provenance policy object");
    let quarantined: BTreeSet<String> = policy
        .get("rights_quarantine")
        .and_then(serde_json::Value::as_object)
        .and_then(|q| q.get("slugs"))
        .and_then(serde_json::Value::as_array)
        .expect("starter provenance quarantine slugs")
        .iter()
        .map(|slug| slug.as_str().expect("quarantine slug string").to_owned())
        .collect();
    let recipes = content
        .get_mut("recipes")
        .and_then(serde_json::Value::as_array_mut)
        .expect("starter provenance recipes");
    recipes.retain(|recipe| {
        let slug = recipe
            .get("slug")
            .and_then(serde_json::Value::as_str)
            .expect("starter recipe slug");
        !quarantined.contains(slug)
    });
    let referenced_catalog: BTreeSet<String> = recipes
        .iter()
        .flat_map(|recipe| {
            recipe
                .get("lines")
                .and_then(serde_json::Value::as_array)
                .expect("starter recipe lines")
        })
        .filter_map(|line| {
            line.get("catalog_id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect();
    content
        .get_mut("catalog")
        .and_then(serde_json::Value::as_array_mut)
        .expect("starter provenance catalog")
        .retain(|ingredient| {
            referenced_catalog.contains(
                ingredient
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .expect("catalog ingredient id"),
            )
        });
    content
        .get_mut("policy")
        .and_then(serde_json::Value::as_object_mut)
        .expect("starter provenance policy object")
        .remove("rights_quarantine");

    let output =
        PathBuf::from(env::var("OUT_DIR").expect("output directory")).join("starter_recipes.json");
    fs::write(
        output,
        serde_json::to_string(&content).expect("serialize shipped starter content"),
    )
    .expect("write shipped starter content");
}
