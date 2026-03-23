use anyhow::{Context, Result};
use clap::Parser;
use dashmap::DashMap;
use jsonschema::{Validator, validator_for};
use rayon::prelude::*;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Path to scan recursively
    path: PathBuf,

    /// Fail if a JSON file has no $schema
    #[arg(long)]
    require_schema: bool,

    /// Print valid files too
    #[arg(long)]
    verbose: bool,

    /// Paths (folder names) to ignore
    #[arg(long, value_name = "NAME")]
    ignore: Vec<String>,
}

use globset::{Glob, GlobSet, GlobSetBuilder};

fn build_ignore_set(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        builder.add(
            Glob::new(pattern).unwrap_or_else(|_| panic!("Invalid glob pattern: {}", pattern)),
        );
    }

    builder.build().expect("Failed to build glob set")
}

fn should_ignore(path: &Path, ignore_set: &GlobSet) -> bool {
    ignore_set.is_match(path)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Default ignores
    // let mut ignore_set: HashSet<String> =
    //     ["target", ".git"].iter().map(|s| s.to_string()).collect();

    // for i in &args.ignore {
    //     ignore_set.insert(i.clone());
    // }

    // let ignore_set = Arc::new(ignore_set);
    let mut patterns = vec!["**/.git/**".to_string(), "**/target/**".to_string()];

    patterns.extend(args.ignore.clone());

    let ignore_set = build_ignore_set(&patterns);

    let cache: DashMap<PathBuf, Validator> = DashMap::new();

    let files: Vec<PathBuf> = WalkDir::new(&args.path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            
            // Strip the base path so that globs can match relative paths exactly,
            // e.g., ".venv/**" will match ".\.venv\..." after stripping ".".
            let relative_path = path.strip_prefix(&args.path).unwrap_or(path);

            if should_ignore(relative_path, &ignore_set) {
                return false;
            }

            path.extension().and_then(|s| s.to_str()) == Some("json")
        })
        .map(|e| e.into_path())
        .collect();

    let had_error = files
        .par_iter()
        .map(|path| {
            process_file(path, &args, &cache)
                .map_err(|e| {
                    eprintln!("❌ {}: {}", path.display(), e);
                })
                .is_err()
        })
        .reduce(|| false, |a, b| a || b);

    if had_error {
        std::process::exit(1);
    }

    Ok(())
}

fn process_file(path: &Path, args: &Args, cache: &DashMap<PathBuf, Validator>) -> Result<()> {
    let text = fs::read_to_string(path).with_context(|| "Failed to read file")?;

    let json: Value = serde_json::from_str(&text).with_context(|| "Invalid JSON")?;

    let schema_str = json.get("$schema").and_then(|v| v.as_str());

    let schema_str = match schema_str {
        Some(s) => s,
        None => {
            if args.require_schema {
                anyhow::bail!("Missing $schema");
            } else {
                return Ok(());
            }
        }
    };

    // Skip spec URLs like draft-07
    if schema_str.starts_with("http://") || schema_str.starts_with("https://") {
        return Ok(());
    }

    let schema_path = resolve_schema_path(path, schema_str);

    let compiled = {
        if let Some(v) = cache.get(&schema_path) {
            v.clone()
        } else {
            let schema_text = fs::read_to_string(&schema_path)
                .with_context(|| format!("Failed to read schema {}", schema_path.display()))?;

            let schema_json: Value = serde_json::from_str(&schema_text)
                .with_context(|| format!("Invalid schema JSON {}", schema_path.display()))?;

            let validator = validator_for(&schema_json)
                .with_context(|| format!("Invalid schema {}", schema_path.display()))?;

            cache.insert(schema_path.clone(), validator.clone());
            validator
        }
    };

    // let compiled = {
    //     // let mut cache = cache.lock().unwrap();
    //     let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());

    //     cache
    //         .entry(schema_path.clone())
    //         .or_insert_with(|| {
    //             let schema_text = fs::read_to_string(&schema_path).expect("Failed to read schema");

    //             let schema_json: Value =
    //                 serde_json::from_str(&schema_text).expect("Invalid schema JSON");

    //             // 👇 THIS is the new API
    //             validator_for(&schema_json).expect("Invalid schema")
    //         })
    //         .clone()
    // };

    if let Err(error) = compiled.validate(&json) {
        println!("❌ {}", path.display());
        // for e in errors. {
        println!("  → {}", error);
        // }
        anyhow::bail!("Validation failed");
    } else if args.verbose {
        println!("✅ {}", path.display());
    }

    Ok(())
}

fn resolve_schema_path(file: &Path, schema: &str) -> PathBuf {
    let base = file.parent().unwrap_or_else(|| Path::new("."));
    base.join(schema)
}
