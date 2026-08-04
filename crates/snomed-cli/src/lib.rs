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
    AssociationRefsetMember, AttributeValueRefsetMember, ComponentAnnotationRefsetMember,
    DescriptionTypeRefsetMember, ExtendedMapRefsetMember, LanguageRefsetMember,
    MemberAnnotationRefsetMember, ModuleDependencyRefsetMember, MrcmAttributeDomainRefsetMember,
    MrcmAttributeRangeRefsetMember, MrcmDomainRefsetMember, MrcmModuleScopeRefsetMember,
    OrderedAssociationRefsetMember, OrderedComponentRefsetMember, OwlExpressionRefsetMember,
    RefsetDescriptorRefsetMember, SimpleMapRefsetMember, SimpleRefsetMember,
};
use snomed_rf2::release_type::ReleaseType;
use snomed_store::{SnapshotStore, SnapshotStoreBuilder};

use snomed_owl::Axiom;

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
        "validate" => cmd_validate(rest),
        "classify" => cmd_classify(rest),
        "nnf" => cmd_nnf(rest),
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
        (
            "export <release-dir> <output-dir> [--full]",
            "convert every exportable file in a release directory to NDJSON",
        ),
        (
            "validate <release-dir> [--full]",
            "check referential integrity and IS-A acyclicity",
        ),
        (
            "classify <release-dir> [concept-id] [--full]",
            "classify the release's OWL axioms; show one concept's entailed supertypes, or a summary",
        ),
        (
            "nnf <release-dir> [concept-id] [--full]",
            "necessary normal form: proximal parents + redundancy-reduced attributes, or a summary",
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

fn cmd_validate(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (dir, release_type) = parse_load_args(args, "validate <release-dir> [--full]")?;
    let (store, mut out) = load(dir, release_type)?;
    let report = store.validate();

    if report.is_clean() {
        writeln!(
            out,
            "no issues found ({} concepts checked)",
            store.concept_count()
        )?;
        return Ok(out);
    }

    writeln!(out, "{} issue(s) found:", report.issue_count())?;
    write_ids(
        &mut out,
        "dangling description concept references",
        &report.dangling_description_concepts,
    )?;
    write_ids(
        &mut out,
        "dangling relationship source references",
        &report.dangling_relationship_sources,
    )?;
    write_ids(
        &mut out,
        "dangling relationship destination references",
        &report.dangling_relationship_destinations,
    )?;
    write_ids(
        &mut out,
        "concepts on a cyclic IS-A path",
        &report.cyclic_concepts,
    )?;
    Ok(out)
}

fn write_ids(out: &mut String, label: &str, ids: &[SctId]) -> Result<(), Box<dyn Error>> {
    if ids.is_empty() {
        return Ok(());
    }
    writeln!(out, "  {label} ({}):", ids.len())?;
    for id in ids {
        writeln!(out, "    {id}")?;
    }
    Ok(())
}

/// Parses every active OWLExpression refset member in the loaded release
/// and runs `snomed-classify`'s EL completion over the result. With a
/// `concept-id`, shows that concept's entailed supertypes (spec/13);
/// without one, a summary. A row that fails to parse (an OWL construct
/// `snomed-owl` doesn't support yet, spec/12) is skipped and reported —
/// same "don't let one bad row block everything else" philosophy as
/// `load`/`validate`, not a hard error.
fn cmd_classify(args: &[String]) -> Result<String, Box<dyn Error>> {
    let usage = "usage: classify <release-dir> [concept-id] [--full]";
    let mut positional: Vec<&str> = Vec::new();
    let mut release_type = ReleaseType::Snapshot;
    for a in args {
        match a.as_str() {
            "--full" => release_type = ReleaseType::Full,
            other => positional.push(other),
        }
    }
    let (dir, concept_id) = match positional.as_slice() {
        [dir] => (*dir, None),
        [dir, id] => (*dir, Some(*id)),
        _ => return Err(usage.into()),
    };

    let (store, mut out) = load(dir, release_type)?;
    let axioms = load_owl_axioms(&store, &mut out)?;

    let report = snomed_classify::classify(&axioms);
    if !report.skipped.is_empty() {
        writeln!(
            out,
            "{} construct(s) not modeled during classification:",
            report.skipped.len()
        )?;
        write_capped(&mut out, &report.skipped, |out, s| writeln!(out, "  {s}"))?;
    }

    match concept_id {
        Some(id_str) => {
            let id = SctId::parse(id_str)?;
            let mut supers: Vec<SctId> = report.classification.subsumers(id).collect();
            supers.sort();
            writeln!(
                out,
                "{id} is entailed to be subsumed by {} concept(s):",
                supers.len()
            )?;
            for s in supers {
                let name = store.fsn(s).map(|d| d.term.as_str()).unwrap_or("?");
                writeln!(out, "  {s}  {name}")?;
            }
        }
        None => {
            let concepts: Vec<SctId> = report.classification.concepts().collect();
            let total_pairs: usize = concepts
                .iter()
                .map(|&c| report.classification.subsumers(c).count())
                .sum();
            writeln!(
                out,
                "{} concept(s) classified, {total_pairs} entailed subsumption pair(s) total",
                concepts.len()
            )?;
        }
    }
    Ok(out)
}

/// Parses every active OWLExpression refset member in `store`, reporting
/// (into `out`) how many parsed versus failed — a row that fails to parse
/// (an OWL construct `snomed-owl` doesn't support yet, spec/12) is
/// skipped and reported, not a hard error, same philosophy as
/// `load`/`validate`. Shared by `classify` and `nnf`, the two subcommands
/// that both start from "every OWL axiom in this release".
fn load_owl_axioms(store: &SnapshotStore, out: &mut String) -> Result<Vec<Axiom>, Box<dyn Error>> {
    let mut axioms = Vec::new();
    let mut parse_failures: Vec<(SctId, String)> = Vec::new();
    for member in store.all_owl_expression_members() {
        match snomed_owl::parse(&member.owl_expression) {
            Ok(axiom) => axioms.push(axiom),
            Err(e) => parse_failures.push((member.core.referenced_component_id, e.to_string())),
        }
    }
    writeln!(
        out,
        "OWL axioms: {} parsed, {} failed to parse",
        axioms.len(),
        parse_failures.len()
    )?;
    write_capped(out, &parse_failures, |out, (id, reason)| {
        writeln!(out, "  parse error on {id}: {reason}")
    })?;
    Ok(axioms)
}

/// Computes the necessary normal form (spec/14) of the release's OWL
/// axioms: proximal (non-redundant) entailed parents, plus role-grouped,
/// redundancy-reduced attributes — built on `snomed-classify`'s
/// classification, one layer up from `classify` itself. With a
/// `concept-id`, shows that concept's form; without one, a summary.
fn cmd_nnf(args: &[String]) -> Result<String, Box<dyn Error>> {
    let usage = "usage: nnf <release-dir> [concept-id] [--full]";
    let mut positional: Vec<&str> = Vec::new();
    let mut release_type = ReleaseType::Snapshot;
    for a in args {
        match a.as_str() {
            "--full" => release_type = ReleaseType::Full,
            other => positional.push(other),
        }
    }
    let (dir, concept_id) = match positional.as_slice() {
        [dir] => (*dir, None),
        [dir, id] => (*dir, Some(*id)),
        _ => return Err(usage.into()),
    };

    let (store, mut out) = load(dir, release_type)?;
    let axioms = load_owl_axioms(&store, &mut out)?;

    let report = snomed_classify::necessary_normal_form(&axioms);
    if !report.skipped.is_empty() {
        writeln!(
            out,
            "{} construct(s) not modeled while computing necessary normal form:",
            report.skipped.len()
        )?;
        write_capped(&mut out, &report.skipped, |out, s| writeln!(out, "  {s}"))?;
    }

    match concept_id {
        Some(id_str) => {
            let id = SctId::parse(id_str)?;
            let name = |id: SctId| {
                store
                    .fsn(id)
                    .map(|d| d.term.as_str())
                    .unwrap_or("?")
                    .to_string()
            };
            match report.forms.get(&id) {
                Some(form) => {
                    writeln!(out, "{id} necessary normal form:")?;
                    writeln!(out, "  is-a ({}):", form.is_a.len())?;
                    for &parent in &form.is_a {
                        writeln!(out, "    {parent}  {}", name(parent))?;
                    }
                    writeln!(out, "  attributes ({}):", form.attributes.len())?;
                    for attr in &form.attributes {
                        writeln!(
                            out,
                            "    group {}: {} ({})  =  {} ({})",
                            attr.group,
                            attr.type_id,
                            name(attr.type_id),
                            attr.destination_id,
                            name(attr.destination_id)
                        )?;
                    }
                }
                None => writeln!(
                    out,
                    "{id}: no necessary normal form (not named by any input axiom)"
                )?,
            }
        }
        None => {
            let concept_count = report.forms.len();
            let total_parents: usize = report.forms.values().map(|f| f.is_a.len()).sum();
            let total_attributes: usize = report.forms.values().map(|f| f.attributes.len()).sum();
            writeln!(
                out,
                "{concept_count} concept(s), {total_parents} proximal parent(s), \
                 {total_attributes} attribute(s) total"
            )?;
        }
    }
    Ok(out)
}

/// Writes at most the first 5 items via `write_one`, then a "... and N
/// more" line if there were more — used for lists that could be large
/// (parse failures, skipped constructs) where dumping every one would
/// swamp the summary this subcommand is meant to give.
fn write_capped<T>(
    out: &mut String,
    items: &[T],
    mut write_one: impl FnMut(&mut String, &T) -> std::fmt::Result,
) -> Result<(), Box<dyn Error>> {
    const CAP: usize = 5;
    for item in items.iter().take(CAP) {
        write_one(out, item)?;
    }
    if items.len() > CAP {
        writeln!(out, "  ... and {} more", items.len() - CAP)?;
    }
    Ok(())
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

/// Dispatches to single-file mode (`export <rf2-file> [output-file]`) or
/// whole-release-directory mode (`export <release-dir> <output-dir>
/// [--full]`), auto-detected by whether the first argument is a directory —
/// so the common single-file shape needs no extra flag.
fn cmd_export(args: &[String]) -> Result<String, Box<dyn Error>> {
    let usage =
        "usage: export <rf2-file> [output-file] | export <release-dir> <output-dir> [--full]";
    let first = args.first().ok_or(usage)?;
    if Path::new(first).is_dir() {
        cmd_export_dir(args)
    } else {
        cmd_export_file(args)
    }
}

/// Converts one RF2 file to NDJSON, dispatching by (content type, summary)
/// exactly like `SnapshotStoreBuilder::load_release_dir`'s internal
/// dispatch — same content types, just serialized instead of stored.
fn cmd_export_file(args: &[String]) -> Result<String, Box<dyn Error>> {
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
    let ndjson = export_to_ndjson(path, &parsed)?.ok_or_else(|| {
        format!(
            "content type `{}` (summary `{}`) is not yet exportable",
            parsed.content_type, parsed.summary
        )
    })?;

    match output {
        Some(out_path) => {
            let line_count = ndjson.lines().count();
            fs::write(out_path, &ndjson)?;
            Ok(format!("wrote {line_count} line(s) to {out_path}\n"))
        }
        None => Ok(ndjson),
    }
}

/// Exports every exportable RF2 file under a release directory in one
/// invocation, mirroring `list_release_files` + per-file dispatch rather
/// than duplicating directory-walking/release-view-filtering logic here —
/// that's real domain logic and belongs in `snomed-store` (see
/// `AGENTS/cli-engineer.md`). One `<file-stem>.ndjson` is written per
/// exported file, flattened into `out_dir` (release file names are unique
/// within one release view, so no collisions). Unsupported content types
/// are skipped and reported, same as `load`; malformed data in a
/// recognized file is a hard error, same as `load`.
fn cmd_export_dir(args: &[String]) -> Result<String, Box<dyn Error>> {
    let usage = "usage: export <release-dir> <output-dir> [--full]";
    let mut positional = Vec::new();
    let mut release_type = ReleaseType::Snapshot;
    for a in args {
        match a.as_str() {
            "--full" => release_type = ReleaseType::Full,
            other => positional.push(other),
        }
    }
    let (dir, out_dir) = match positional.as_slice() {
        [dir, out_dir] => (*dir, *out_dir),
        _ => return Err(usage.into()),
    };

    let files = snomed_store::list_release_files(Path::new(dir), release_type)?;
    fs::create_dir_all(out_dir)?;

    let mut exported = 0usize;
    let mut skipped: Vec<(std::path::PathBuf, String)> = Vec::new();
    for (path, parsed) in &files {
        match export_to_ndjson(path, parsed)? {
            Some(ndjson) => {
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or("input file name is not valid UTF-8")?;
                fs::write(Path::new(out_dir).join(format!("{stem}.ndjson")), &ndjson)?;
                exported += 1;
            }
            None => skipped.push((
                path.clone(),
                format!(
                    "content type `{}` (summary `{}`) is not yet exportable",
                    parsed.content_type, parsed.summary
                ),
            )),
        }
    }

    let mut out = String::new();
    writeln!(
        out,
        "exported {exported} file(s), skipped {} to {out_dir}",
        skipped.len()
    )?;
    for (path, reason) in &skipped {
        writeln!(out, "  skipped {}: {reason}", path.display())?;
    }
    Ok(out)
}

/// `Ok(None)` means the (content type, summary) combination isn't wired up
/// for export yet — a skip, not an error (mirrors `load.rs::dispatch`'s
/// `Ok(Some(reason))` shape for the same distinction). `Err` is reserved
/// for genuine I/O/RF2-parsing failure on a file this function recognized.
fn export_to_ndjson(path: &Path, f: &ReleaseFileName) -> Result<Option<String>, Box<dyn Error>> {
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
        ("cRefset", "MRCMModuleScope") => export_rows::<MrcmModuleScopeRefsetMember, _>(
            path,
            &mut out,
            json::mrcm_module_scope_refset_to_json,
        )?,
        ("sssssssRefset", "MRCMDomain") => export_rows::<MrcmDomainRefsetMember, _>(
            path,
            &mut out,
            json::mrcm_domain_refset_to_json,
        )?,
        ("cissccRefset", "MRCMAttributeDomain") => {
            export_rows::<MrcmAttributeDomainRefsetMember, _>(
                path,
                &mut out,
                json::mrcm_attribute_domain_refset_to_json,
            )?
        }
        ("ssccRefset", "MRCMAttributeRange") => export_rows::<MrcmAttributeRangeRefsetMember, _>(
            path,
            &mut out,
            json::mrcm_attribute_range_refset_to_json,
        )?,
        ("iRefset", "OrderedComponent") => export_rows::<OrderedComponentRefsetMember, _>(
            path,
            &mut out,
            json::ordered_component_refset_to_json,
        )?,
        ("ciRefset", "OrderedAssociation") => export_rows::<OrderedAssociationRefsetMember, _>(
            path,
            &mut out,
            json::ordered_association_refset_to_json,
        )?,
        ("scsRefset", "ComponentAnnotationStringValue") => {
            export_rows::<ComponentAnnotationRefsetMember, _>(
                path,
                &mut out,
                json::component_annotation_refset_to_json,
            )?
        }
        ("sscsRefset", "MemberAnnotationStringValue") => {
            export_rows::<MemberAnnotationRefsetMember, _>(
                path,
                &mut out,
                json::member_annotation_refset_to_json,
            )?
        }
        (_, _) => return Ok(None),
    }
    Ok(Some(out))
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
