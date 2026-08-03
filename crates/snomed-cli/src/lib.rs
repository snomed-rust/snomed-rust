//! Command-line toolkit over the `snomed` workspace crates: SCTID
//! validation, release loading, concept lookup, ECL queries.
//!
//! This crate is deliberately a thin presentation layer — every subcommand
//! is a few lines of formatting around calls into `snomed-core`,
//! `snomed-rf2`, `snomed-store`, and `snomed-ecl`. New domain logic belongs
//! in those crates, not here (see `AGENTS/cli-engineer.md`).
//!
//! [`run`] is the single entry point, returning the formatted output as a
//! `String` (rather than printing directly) so subcommands are unit- and
//! integration-testable without spawning the compiled binary.

mod json;

use std::error::Error;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};
use snomed_core::sctid::SctId;
use snomed_rf2::filename::ReleaseFileName;
use snomed_rf2::reader::Rf2Reader;
use snomed_rf2::record::Rf2Record;
use snomed_rf2::refset::{
    AssociationRefsetMember, AttributeValueRefsetMember, DescriptionTypeRefsetMember,
    ExtendedMapRefsetMember, LanguageRefsetMember, ModuleDependencyRefsetMember,
    OwlExpressionRefsetMember, RefsetDescriptorRefsetMember, SimpleMapRefsetMember,
    SimpleRefsetMember,
};
use snomed_rf2::release_type::ReleaseType;
use snomed_store::{SnapshotStore, SnapshotStoreBuilder};

/// Dispatches on `args[0]` (the subcommand name) and returns the formatted
/// output. `args` excludes the program name (pass `std::env::args().skip(1)`
/// collected into a `Vec`, or an equivalent slice in tests).
pub fn run(args: &[String]) -> Result<String, Box<dyn Error>> {
    let Some((cmd, rest)) = args.split_first() else {
        return Ok(usage());
    };
    match cmd.as_str() {
        "sctid" => cmd_sctid(rest),
        "load" => cmd_load(rest),
        "lookup" => cmd_lookup(rest),
        "ecl" => cmd_ecl(rest),
        "export" => cmd_export(rest),
        "help" | "-h" | "--help" => Ok(usage()),
        other => Err(format!("unknown command `{other}` (try `snomed-cli help`)").into()),
    }
}

fn usage() -> String {
    let rows: &[(&str, &str)] = &[
        ("sctid <id>", "validate an SCTID and show its structure"),
        (
            "load <release-dir> [--full]",
            "load a release directory, print a summary",
        ),
        (
            "lookup <release-dir> <id>",
            "look up a concept: FSN, synonyms, parents, children",
        ),
        (
            "ecl <release-dir> <expression>",
            "evaluate an ECL expression (quote it)",
        ),
        (
            "export <rf2-file> [output-file]",
            "convert one RF2 file to NDJSON (stdout if no output file)",
        ),
    ];
    let width = rows.iter().map(|(cmd, _)| cmd.len()).max().unwrap_or(0);

    let mut out = String::new();
    let _ = writeln!(out, "snomed-cli — local SNOMED CT RF2 toolkit\n");
    let _ = writeln!(out, "USAGE:");
    for (cmd, desc) in rows {
        let _ = writeln!(out, "  snomed-cli {cmd:width$}   {desc}");
    }
    let _ = writeln!(
        out,
        "\n<release-dir> is an unzipped RF2 release directory. `load`/`lookup`/`ecl`\n\
         read its Snapshot view by default; `load --full` reads the Full view."
    );
    out
}

fn cmd_sctid(args: &[String]) -> Result<String, Box<dyn Error>> {
    let raw = args.first().ok_or("usage: sctid <id>")?;
    let id = SctId::parse(raw)?;

    let mut out = String::new();
    writeln!(out, "{id}")?;
    writeln!(
        out,
        "  component type: {}",
        id.component_type()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    )?;
    writeln!(
        out,
        "  format:         {}",
        if id.is_long_format() {
            "long (extension)"
        } else {
            "short (International)"
        }
    )?;
    writeln!(out, "  partition:      {:02}", id.partition())?;
    if let Some(ns) = id.namespace() {
        writeln!(out, "  namespace:      {ns:07}")?;
    }
    writeln!(out, "  item id:        {}", id.item_identifier())?;
    writeln!(out, "  check digit:    {}", id.check_digit())?;
    Ok(out)
}

fn parse_load_args<'a>(
    args: &'a [String],
    usage_msg: &'static str,
) -> Result<(&'a str, ReleaseType), Box<dyn Error>> {
    let mut dir = None;
    let mut release_type = ReleaseType::Snapshot;
    for a in args {
        match a.as_str() {
            "--full" => release_type = ReleaseType::Full,
            other if dir.is_none() => dir = Some(other),
            other => {
                return Err(format!("unexpected argument `{other}`\nusage: {usage_msg}").into())
            }
        }
    }
    let dir = dir.ok_or_else(|| format!("usage: {usage_msg}"))?;
    Ok((dir, release_type))
}

fn load(dir: &str, release_type: ReleaseType) -> Result<(SnapshotStore, String), Box<dyn Error>> {
    let start = Instant::now();
    let mut builder = SnapshotStoreBuilder::new();
    let report = builder.load_release_dir(Path::new(dir), release_type)?;
    let elapsed = start.elapsed();

    let mut out = String::new();
    writeln!(
        out,
        "loaded {} file(s), skipped {} in {elapsed:.2?}",
        report.loaded.len(),
        report.skipped.len()
    )?;
    for (path, reason) in &report.skipped {
        writeln!(out, "  skipped {}: {reason}", path.display())?;
    }
    let store = builder.build();
    Ok((store, out))
}

fn cmd_load(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (dir, release_type) = parse_load_args(args, "load <release-dir> [--full]")?;
    let (store, mut out) = load(dir, release_type)?;
    writeln!(
        out,
        "concepts: {} ({} active)",
        store.concept_count(),
        store.active_concepts().count()
    )?;
    Ok(out)
}

fn cmd_lookup(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (dir, id_raw) = match args {
        [dir, id] => (dir.as_str(), id.as_str()),
        _ => return Err("usage: lookup <release-dir> <id>".into()),
    };
    let id = SctId::parse(id_raw)?;
    let (store, _) = load(dir, ReleaseType::Snapshot)?;

    let mut out = String::new();
    let Some(concept) = store.concept(id) else {
        writeln!(out, "{id}: not found in this snapshot")?;
        return Ok(out);
    };
    writeln!(
        out,
        "{id}  active={}  module={}",
        concept.active, concept.module_id
    )?;
    if let Some(fsn) = store.fsn(id) {
        writeln!(out, "  FSN: {}", fsn.term)?;
    }
    for syn in store
        .descriptions_of(id)
        .filter(|d| d.active && d.is_synonym())
    {
        writeln!(out, "  synonym: {}", syn.term)?;
    }
    write_related(&mut out, "parents", store.parents(id), &store)?;
    write_related(&mut out, "children", store.children(id), &store)?;
    Ok(out)
}

fn write_related(
    out: &mut String,
    label: &str,
    ids: &[SctId],
    store: &SnapshotStore,
) -> Result<(), Box<dyn Error>> {
    if ids.is_empty() {
        return Ok(());
    }
    writeln!(out, "  {label}:")?;
    for &id in ids {
        let name = store.fsn(id).map(|d| d.term.as_str()).unwrap_or("?");
        writeln!(out, "    {id}  {name}")?;
    }
    Ok(())
}

fn cmd_ecl(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (dir, expr_str) = match args {
        [dir, expr] => (dir.as_str(), expr.as_str()),
        _ => return Err("usage: ecl <release-dir> <expression> (quote the expression)".into()),
    };
    let (store, _) = load(dir, ReleaseType::Snapshot)?;

    let expr = snomed_ecl::parse(expr_str)?;
    let matches = snomed_ecl::evaluate(&expr, &store);
    let mut sorted: Vec<SctId> = matches.into_iter().collect();
    sorted.sort();

    let mut out = String::new();
    writeln!(out, "{} match(es)", sorted.len())?;
    for id in sorted {
        let name = store.fsn(id).map(|d| d.term.as_str()).unwrap_or("?");
        writeln!(out, "{id}  {name}")?;
    }
    Ok(out)
}

/// Converts one RF2 file to NDJSON, dispatching by (content type, summary)
/// exactly like `SnapshotStoreBuilder::load_release_dir`'s internal
/// dispatch — same content types, just serialized instead of stored.
fn cmd_export(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (input, output) = match args {
        [input] => (input.as_str(), None),
        [input, output] => (input.as_str(), Some(output.as_str())),
        _ => return Err("usage: export <rf2-file> [output-file]".into()),
    };

    let path = Path::new(input);
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("input file name is not valid UTF-8")?;
    let parsed = ReleaseFileName::parse(file_name)?;
    let ndjson = export_to_ndjson(path, &parsed)?;

    match output {
        Some(out_path) => {
            let line_count = ndjson.lines().count();
            fs::write(out_path, &ndjson)?;
            Ok(format!("wrote {line_count} line(s) to {out_path}\n"))
        }
        None => Ok(ndjson),
    }
}

fn export_to_ndjson(path: &Path, f: &ReleaseFileName) -> Result<String, Box<dyn Error>> {
    let mut out = String::new();
    match (f.content_type.as_str(), f.summary.as_str()) {
        ("Concept", _) => export_rows::<Concept, _>(path, &mut out, json::concept_to_json)?,
        ("Description", _) | ("TextDefinition", _) => {
            export_rows::<Description, _>(path, &mut out, json::description_to_json)?
        }
        ("Relationship", _) | ("StatedRelationship", _) => {
            export_rows::<Relationship, _>(path, &mut out, json::relationship_to_json)?
        }
        ("RelationshipConcreteValues", _) => export_rows::<RelationshipConcreteValue, _>(
            path,
            &mut out,
            json::relationship_concrete_value_to_json,
        )?,
        ("Refset", _) => {
            export_rows::<SimpleRefsetMember, _>(path, &mut out, json::simple_refset_to_json)?
        }
        ("cRefset", "Language") => {
            export_rows::<LanguageRefsetMember, _>(path, &mut out, json::language_refset_to_json)?
        }
        ("cRefset", summary) if summary.contains("Association") => {
            export_rows::<AssociationRefsetMember, _>(
                path,
                &mut out,
                json::association_refset_to_json,
            )?
        }
        ("cRefset", summary) if summary.contains("AttributeValue") => {
            export_rows::<AttributeValueRefsetMember, _>(
                path,
                &mut out,
                json::attribute_value_refset_to_json,
            )?
        }
        ("sRefset", "SimpleMap") => export_rows::<SimpleMapRefsetMember, _>(
            path,
            &mut out,
            json::simple_map_refset_to_json,
        )?,
        ("sRefset", "OWLExpression") => export_rows::<OwlExpressionRefsetMember, _>(
            path,
            &mut out,
            json::owl_expression_refset_to_json,
        )?,
        ("iisssccRefset", _) => export_rows::<ExtendedMapRefsetMember, _>(
            path,
            &mut out,
            json::extended_map_refset_to_json,
        )?,
        ("ssRefset", "ModuleDependency") => export_rows::<ModuleDependencyRefsetMember, _>(
            path,
            &mut out,
            json::module_dependency_refset_to_json,
        )?,
        ("cciRefset", "RefsetDescriptor") => export_rows::<RefsetDescriptorRefsetMember, _>(
            path,
            &mut out,
            json::refset_descriptor_refset_to_json,
        )?,
        ("ciRefset", "DescriptionType") => export_rows::<DescriptionTypeRefsetMember, _>(
            path,
            &mut out,
            json::description_type_refset_to_json,
        )?,
        (content_type, summary) => {
            return Err(format!(
                "content type `{content_type}` (summary `{summary}`) is not yet exportable"
            )
            .into())
        }
    }
    Ok(out)
}

fn export_rows<T, F>(path: &Path, out: &mut String, to_json: F) -> Result<(), Box<dyn Error>>
where
    T: Rf2Record,
    F: Fn(&T) -> String,
{
    let file = File::open(path)?;
    let reader = Rf2Reader::<_, T>::new(BufReader::new(file))?;
    for row in reader {
        out.push_str(&to_json(&row?));
        out.push('\n');
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_args_prints_usage() {
        let out = run(&[]).unwrap();
        assert!(out.contains("USAGE"));
    }

    #[test]
    fn help_prints_usage() {
        let out = run(&args(&["help"])).unwrap();
        assert!(out.contains("USAGE"));
    }

    #[test]
    fn unknown_command_errors() {
        let err = run(&args(&["nope"])).unwrap_err();
        assert!(err.to_string().contains("unknown command"));
    }

    #[test]
    fn sctid_reports_structure() {
        let out = run(&args(&["sctid", "138875005"])).unwrap();
        assert!(out.contains("component type: Concept"));
        assert!(out.contains("short (International)"));
    }

    #[test]
    fn sctid_rejects_malformed_input() {
        let err = run(&args(&["sctid", "not-an-id"])).unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn load_missing_dir_errors() {
        let err = run(&args(&["load"])).unwrap_err();
        assert!(err.to_string().contains("usage"));
    }
}
