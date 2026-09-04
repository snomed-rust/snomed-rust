//! Every `spec/NN rule M` citation in the repository names a rule that
//! exists.
//!
//! Code and docs cite spec rules by number (CLAUDE.md rule 1), and rule
//! lists get renumbered when a rule is inserted — twice in one day, while
//! this workspace's ECL and benchmark specs were being extended. A stale
//! citation is invisible: it reads correctly and points at the wrong rule,
//! or at none. This walks the repository and checks them all.
//!
//! It is a test rather than a script so `cargo test` runs it, needing no
//! tool the workspace doesn't already require.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The repository root, or `None` when this test runs from a packaged
/// crate (where `spec/` isn't shipped) — in which case there is nothing to
/// check and the test passes.
fn repo_root() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    root.join("spec").is_dir().then_some(root)
}

/// Top-level numbered list items in a markdown file: `1. `, `2. `, …
fn numbered_items(text: &str) -> HashSet<u32> {
    text.lines()
        .filter_map(|line| {
            let digits: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
            let rest = &line[digits.len()..];
            (!digits.is_empty() && rest.starts_with(". "))
                .then(|| digits.parse().ok())
                .flatten()
        })
        .collect()
}

fn markdown_and_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Skip build output and the verbatim archives, whose citations
        // were correct when written and are history now.
        if name == "target" || name == ".git" || name.contains("archive") {
            continue;
        }
        if path.is_dir() {
            markdown_and_rust_files(&path, out);
        } else if matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("md") | Some("rs")
        ) {
            out.push(path);
        }
    }
}

/// `spec/09` -> `09-versioning.md`; `spec/rust-bench.md` -> itself. A bare
/// number resolves to the shortest matching file, so `spec/10` is
/// `10-ecl.md` rather than the split-off `10-ecl-filters.md`.
fn resolve<'a>(token: &str, specs: &'a HashMap<String, HashSet<u32>>) -> Option<&'a str> {
    let token = token.trim_end_matches(['`', ',', '\'', 's']);
    if let Some((name, _)) = specs.get_key_value(token) {
        return Some(name);
    }
    let with_ext = format!("{token}.md");
    if specs.contains_key(&with_ext) {
        return specs.get_key_value(&with_ext).map(|(k, _)| k.as_str());
    }
    specs
        .keys()
        .filter(|name| name.starts_with(&format!("{token}-")))
        .min_by_key(|name| name.len())
        .map(String::as_str)
}

#[test]
fn every_spec_rule_citation_names_a_rule_that_exists() {
    let Some(root) = repo_root() else { return };

    let mut specs: HashMap<String, HashSet<u32>> = HashMap::new();
    for entry in fs::read_dir(root.join("spec"))
        .expect("spec/ is readable")
        .flatten()
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            specs.insert(name, numbered_items(&fs::read_to_string(&path).unwrap()));
        }
    }
    assert!(specs.len() > 10, "found {} spec files", specs.len());

    let mut files = Vec::new();
    markdown_and_rust_files(&root, &mut files);

    let mut checked = 0;
    let mut problems = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_default();
        // Find `spec/<token>` and the first `rule <n>` within the next few
        // words — the shape citations take throughout this repository.
        for (index, _) in text.match_indices("spec/") {
            let tail = &text[index + 5..];
            let token: String = tail
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
                .collect();
            let token = token.trim_end_matches('.').to_string();
            let window: String = tail.chars().take(token.len() + 30).collect();
            let Some(rule_at) = window.find(" rule ") else {
                continue;
            };
            let number: String = window[rule_at + 6..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            let Ok(rule) = number.parse::<u32>() else {
                continue;
            };
            // A citation may name a rule the *other* spec in the same
            // sentence defines; only flag when the resolved file is
            // unambiguous.
            if window[..rule_at].contains("spec/") {
                continue;
            }
            checked += 1;
            match resolve(&token, &specs) {
                None => problems.push(format!("{}: unknown spec `{token}`", file.display())),
                // Rule 0 exists in spec/10 and sorts before 1.
                Some(name) if rule != 0 && !specs[name].contains(&rule) => problems.push(format!(
                    "{}: cites `spec/{token} rule {rule}`, but {name} defines rules {:?}",
                    file.display(),
                    {
                        let mut v: Vec<_> = specs[name].iter().copied().collect();
                        v.sort_unstable();
                        v
                    }
                )),
                _ => {}
            }
        }
    }

    assert!(
        checked > 50,
        "expected the repository to cite many rules, found {checked}"
    );
    assert!(
        problems.is_empty(),
        "stale spec citations:\n  {}",
        problems.join("\n  ")
    );
}
