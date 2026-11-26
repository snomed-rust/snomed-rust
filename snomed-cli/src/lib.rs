//! Command-line toolkit over the `snomed` workspace crates: SCTID
//! validation, release loading, concept lookup, ECL queries.
//!
//! This crate is deliberately a thin presentation layer — every subcommand
//! is a few lines of formatting around calls into `snomed-core`,
//! `snomed-rf2`, `snomed-store`, and `snomed-ecl`. New domain logic belongs
//! in those crates, not here (see `agents/cli-engineer.md`).
//!
//! [`run`] is the single entry point, returning the formatted output as a
//! `String` (rather than printing directly) so subcommands are unit- and
//! integration-testable without spawning the compiled binary.
//!
//! # Trademarks
//!
//! SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
//! International Health Terminology Standards Development Organisation
//! (IHTSDO). Use of the trademarks does not constitute endorsement of this
//! product by IHTSDO. This project is an independent work: it is not
//! affiliated with, endorsed by, or certified by SNOMED International, and it
//! ships no SNOMED CT content.

#![forbid(unsafe_code)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(coverage_nightly, allow(unused_features))]
// Per spec/rust-no-unsafe/index.md: this workspace contains no `unsafe`, and
// the compiler enforces that rather than a grep.

mod json;

use std::error::Error;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;

use snomed_core::components::{
    AlternateIdentifier, Concept, Description, Relationship, RelationshipConcreteValue,
};
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
    let _ = writeln!(out, "{id}");
    let component_type = component_type_label(id);
    let _ = writeln!(out, "  component type: {}", component_type);
    let _ = writeln!(
        out,
        "  format:         {}",
        if id.is_long_format() {
            "long (extension)"
        } else {
            "short (International)"
        }
    );
    let _ = writeln!(out, "  partition:      {:02}", id.partition());
    if let Some(ns) = id.namespace() {
        let _ = writeln!(out, "  namespace:      {ns:07}");
    }
    let _ = writeln!(out, "  item id:        {}", id.item_identifier());
    let _ = writeln!(out, "  check digit:    {}", id.check_digit());
    Ok(out)
}

fn component_type_label(id: SctId) -> String {
    match id.component_type() {
        Some(component_type) => component_type.to_string(),
        None => "unknown".to_string(),
    }
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
    let _ = writeln!(
        out,
        "loaded {} file(s), skipped {} in {elapsed:.2?}",
        report.loaded.len(),
        report.skipped.len()
    );
    for (path, reason) in &report.skipped {
        let _ = writeln!(out, "  skipped {}: {reason}", path.display());
    }
    let store = builder.build();
    Ok((store, out))
}

fn cmd_load(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (dir, release_type) = parse_load_args(args, "load <release-dir> [--full]")?;
    let (store, mut out) = load(dir, release_type)?;
    let _ = writeln!(
        out,
        "concepts: {} ({} active)",
        store.concept_count(),
        store.active_concepts().count()
    );
    Ok(out)
}

/// `validate` found issues, or found nothing to check: the complete report
/// is in `report`. The binary prints it to stdout (like a clean report) and
/// exits non-zero, so a release pipeline cannot pass a bad or empty release.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationFailed {
    /// The full `validate` output, exactly as a clean run would print it.
    pub report: String,
}

impl std::fmt::Display for ValidationFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("validation failed")
    }
}

impl Error for ValidationFailed {}

fn cmd_validate(args: &[String]) -> Result<String, Box<dyn Error>> {
    let (dir, release_type) = parse_load_args(args, "validate <release-dir> [--full]")?;
    let (store, mut out) = load(dir, release_type)?;
    let report = store.validate();

    if store.concept_count() == 0 {
        let _ = writeln!(out, "no concepts loaded: nothing to validate");
        return Err(Box::new(ValidationFailed { report: out }));
    }
    if report.is_clean() {
        let _ = writeln!(
            out,
            "no issues found ({} concepts checked)",
            store.concept_count()
        );
        return Ok(out);
    }

    let _ = writeln!(out, "{} issue(s) found:", report.issue_count());
    write_ids(
        &mut out,
        "dangling description concept references",
        &report.dangling_description_concepts,
    );
    write_ids(
        &mut out,
        "dangling relationship source references",
        &report.dangling_relationship_sources,
    );
    write_ids(
        &mut out,
        "dangling relationship destination references",
        &report.dangling_relationship_destinations,
    );
    write_ids(
        &mut out,
        "concepts on a cyclic IS-A path",
        &report.cyclic_concepts,
    );
    write_ids(
        &mut out,
        "active concepts with no IS-A parent (spec/07 rule 2)",
        &report.rootless_concepts,
    );
    if !report.unsatisfied_module_dependencies.is_empty() {
        let _ = writeln!(
            out,
            "  module dependencies whose target module is not loaded at the required version (spec/08 rule 5) ({}):",
            report.unsatisfied_module_dependencies.len()
        );
        for id in &report.unsatisfied_module_dependencies {
            let _ = writeln!(out, "    {id}");
        }
    }
    Err(Box::new(ValidationFailed { report: out }))
}

fn write_ids(out: &mut String, label: &str, ids: &[SctId]) {
    if ids.is_empty() {
        return;
    }
    let _ = writeln!(out, "  {label} ({}):", ids.len());
    for id in ids {
        let _ = writeln!(out, "    {id}");
    }
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
    let axioms = load_owl_axioms(&store, &mut out);

    let report = snomed_classify::classify(&axioms);
    if !report.skipped.is_empty() {
        let _ = writeln!(
            out,
            "{} construct(s) not modeled during classification:",
            report.skipped.len()
        );
        write_capped(&mut out, &report.skipped, write_skipped_construct);
    }

    match concept_id {
        Some(id_str) => {
            let id = SctId::parse(id_str)?;
            let mut supers: Vec<SctId> = report.classification.subsumers(id).collect();
            supers.sort();
            let _ = writeln!(
                out,
                "{id} is entailed to be subsumed by {} concept(s):",
                supers.len()
            );
            for s in supers {
                let name = concept_name(&store, s);
                let _ = writeln!(out, "  {s}  {name}");
            }
        }
        None => {
            let concepts: Vec<SctId> = report.classification.concepts().collect();
            let mut total_pairs = 0usize;
            for &concept in &concepts {
                total_pairs += report.classification.subsumers(concept).count();
            }
            let _ = writeln!(
                out,
                "{} concept(s) classified, {total_pairs} entailed subsumption pair(s) total",
                concepts.len()
            );
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
fn load_owl_axioms(store: &SnapshotStore, out: &mut String) -> Vec<Axiom> {
    let mut axioms = Vec::new();
    let mut parse_failures: Vec<(SctId, String)> = Vec::new();
    for member in store.all_owl_expression_members() {
        match snomed_owl::parse(&member.owl_expression) {
            Ok(axiom) => axioms.push(axiom),
            Err(e) => parse_failures.push((member.core.referenced_component_id, e.to_string())),
        }
    }
    let _ = writeln!(
        out,
        "OWL axioms: {} parsed, {} failed to parse",
        axioms.len(),
        parse_failures.len()
    );
    write_capped(out, &parse_failures, write_parse_failure);
    axioms
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
    let axioms = load_owl_axioms(&store, &mut out);

    let report = snomed_classify::necessary_normal_form(&axioms);
    if !report.skipped.is_empty() {
        let _ = writeln!(
            out,
            "{} construct(s) not modeled while computing necessary normal form:",
            report.skipped.len()
        );
        write_capped(&mut out, &report.skipped, write_skipped_construct);
    }

    match concept_id {
        Some(id_str) => {
            let id = SctId::parse(id_str)?;
            match report.forms.get(&id) {
                Some(form) => {
                    let _ = writeln!(out, "{id} necessary normal form:");
                    let _ = writeln!(out, "  is-a ({}):", form.is_a.len());
                    for &parent in &form.is_a {
                        let _ = writeln!(out, "    {parent}  {}", concept_name(&store, parent));
                    }
                    let _ = writeln!(out, "  attributes ({}):", form.attributes.len());
                    for attr in &form.attributes {
                        let _ = writeln!(
                            out,
                            "    group {}: {} ({})  =  {} ({})",
                            attr.group,
                            attr.type_id,
                            concept_name(&store, attr.type_id),
                            attr.destination_id,
                            concept_name(&store, attr.destination_id)
                        );
                    }
                }
                None => {
                    let _ = writeln!(
                        out,
                        "{id}: no necessary normal form (not named by any input axiom)"
                    );
                }
            }
        }
        None => {
            let concept_count = report.forms.len();
            let mut total_parents = 0usize;
            let mut total_attributes = 0usize;
            for form in report.forms.values() {
                total_parents += form.is_a.len();
                total_attributes += form.attributes.len();
            }
            let _ = writeln!(
                out,
                "{concept_count} concept(s), {total_parents} proximal parent(s), \
                 {total_attributes} attribute(s) total"
            );
        }
    }
    Ok(out)
}

/// Writes at most the first 5 items via `write_one`, then a "... and N
/// more" line if there were more — used for lists that could be large
/// (parse failures, skipped constructs) where dumping every one would
/// swamp the summary this subcommand is meant to give.
fn write_capped<T>(out: &mut String, items: &[T], write_one: fn(&mut String, &T)) {
    const CAP: usize = 5;
    for item in items.iter().take(CAP) {
        write_one(out, item);
    }
    if let Some(rest) = items.get(CAP..).filter(|rest| !rest.is_empty()) {
        let _ = writeln!(out, "  ... and {} more", rest.len());
    }
}

fn write_skipped_construct(out: &mut String, skipped: &snomed_classify::SkippedConstruct) {
    let _ = writeln!(out, "  {skipped}");
}

fn write_parse_failure(out: &mut String, failure: &(SctId, String)) {
    let (id, reason) = failure;
    let _ = writeln!(out, "  parse error on {id}: {reason}");
}

fn concept_name(store: &SnapshotStore, id: SctId) -> &str {
    match store.fsn(id) {
        Some(description) => description.term.as_str(),
        None => "?",
    }
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
        let _ = writeln!(out, "{id}: not found in this snapshot");
        return Ok(out);
    };
    let _ = writeln!(
        out,
        "{id}  active={}  module={}",
        concept.active, concept.module_id
    );
    if let Some(fsn) = store.fsn(id) {
        let _ = writeln!(out, "  FSN: {}", fsn.term);
    }
    for description in store.descriptions_of(id) {
        if description.active && description.is_synonym() {
            let _ = writeln!(out, "  synonym: {}", description.term);
        }
    }
    write_related(&mut out, "parents", store.parents(id), &store);
    write_related(&mut out, "children", store.children(id), &store);
    Ok(out)
}

fn write_related(out: &mut String, label: &str, ids: &[SctId], store: &SnapshotStore) {
    if ids.is_empty() {
        return;
    }
    let _ = writeln!(out, "  {label}:");
    for &id in ids {
        let name = concept_name(store, id);
        let _ = writeln!(out, "    {id}  {name}");
    }
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
    let _ = writeln!(out, "{} match(es)", sorted.len());
    for id in sorted {
        let name = concept_name(&store, id);
        let _ = writeln!(out, "{id}  {name}");
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
    let file_name = utf8_file_name(path)?;
    let parsed = ReleaseFileName::parse(file_name)?;
    let ndjson = match export_to_ndjson(path, &parsed)? {
        Some(ndjson) => ndjson,
        None => {
            return Err(format!(
                "content type `{}` (summary `{}`) is not yet exportable",
                parsed.content_type, parsed.summary
            )
            .into())
        }
    };

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
/// `agents/cli-engineer.md`). One `<file-stem>.ndjson` is written per
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

    // Two inputs with the same stem (the same file in two subdirectories)
    // would write the same output path, and the second would silently
    // replace the first while both counted as exported. Refuse before
    // writing anything.
    let mut outputs: std::collections::BTreeMap<String, &Path> = std::collections::BTreeMap::new();
    for (path, parsed) in &files {
        if let Some(first) = outputs.insert(parsed.file_stem(), path) {
            return Err(format!(
                "{} and {} would both export to {}.ndjson",
                first.display(),
                path.display(),
                parsed.file_stem()
            )
            .into());
        }
    }

    let mut exported = 0usize;
    let mut skipped: Vec<(std::path::PathBuf, String)> = Vec::new();
    for (path, parsed) in &files {
        match export_to_ndjson(path, parsed)? {
            Some(ndjson) => {
                let stem = parsed.file_stem();
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
    let _ = writeln!(
        out,
        "exported {exported} file(s), skipped {} to {out_dir}",
        skipped.len()
    );
    for (path, reason) in &skipped {
        let _ = writeln!(out, "  skipped {}: {reason}", path.display());
    }
    Ok(out)
}

fn utf8_file_name(path: &Path) -> Result<&str, Box<dyn Error>> {
    match path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => Ok(name),
            None => Err("input file name is not valid UTF-8".into()),
        },
        None => Err("input file name is not valid UTF-8".into()),
    }
}

/// `Ok(None)` means the (content type, summary) combination isn't wired up
/// for export yet — a skip, not an error (mirrors `load.rs::dispatch`'s
/// `Ok(Some(reason))` shape for the same distinction). `Err` is reserved
/// for genuine I/O/RF2-parsing failure on a file this function recognized.
fn export_to_ndjson(path: &Path, f: &ReleaseFileName) -> Result<Option<String>, Box<dyn Error>> {
    let mut out = String::new();
    macro_rules! export {
        ($record:ty, $to_json:path) => {
            return finish_export(export_rows::<$record, _>(path, &mut out, $to_json), out)
        };
    }
    match (f.content_type.as_str(), f.summary.as_str()) {
        ("Concept", _) => export!(Concept, json::concept_to_json),
        ("Description", _) | ("TextDefinition", _) => {
            export!(Description, json::description_to_json)
        }
        ("Relationship", _) | ("StatedRelationship", _) => {
            export!(Relationship, json::relationship_to_json)
        }
        ("RelationshipConcreteValues", _) => export!(
            RelationshipConcreteValue,
            json::relationship_concrete_value_to_json
        ),
        ("Identifier", _) => export!(AlternateIdentifier, json::alternate_identifier_to_json),
        ("Refset", _) => export!(SimpleRefsetMember, json::simple_refset_to_json),
        ("cRefset", "Language") => export!(LanguageRefsetMember, json::language_refset_to_json),
        ("cRefset", summary) if summary.contains("Association") => {
            export!(AssociationRefsetMember, json::association_refset_to_json)
        }
        ("cRefset", summary) if summary.contains("AttributeValue") => {
            export!(
                AttributeValueRefsetMember,
                json::attribute_value_refset_to_json
            )
        }
        ("sRefset", "SimpleMap") => export!(SimpleMapRefsetMember, json::simple_map_refset_to_json),
        ("sRefset", "OWLExpression") => {
            export!(
                OwlExpressionRefsetMember,
                json::owl_expression_refset_to_json
            )
        }
        ("iisssccRefset", _) => {
            export!(ExtendedMapRefsetMember, json::extended_map_refset_to_json)
        }
        ("ssRefset", "ModuleDependency") => export!(
            ModuleDependencyRefsetMember,
            json::module_dependency_refset_to_json
        ),
        ("cciRefset", "RefsetDescriptor") => export!(
            RefsetDescriptorRefsetMember,
            json::refset_descriptor_refset_to_json
        ),
        ("ciRefset", "DescriptionType") => export!(
            DescriptionTypeRefsetMember,
            json::description_type_refset_to_json
        ),
        ("cRefset", "MRCMModuleScope") => export!(
            MrcmModuleScopeRefsetMember,
            json::mrcm_module_scope_refset_to_json
        ),
        ("sssssssRefset", "MRCMDomain") => {
            export!(MrcmDomainRefsetMember, json::mrcm_domain_refset_to_json)
        }
        ("cissccRefset", "MRCMAttributeDomain") => {
            export!(
                MrcmAttributeDomainRefsetMember,
                json::mrcm_attribute_domain_refset_to_json
            )
        }
        ("ssccRefset", "MRCMAttributeRange") => export!(
            MrcmAttributeRangeRefsetMember,
            json::mrcm_attribute_range_refset_to_json
        ),
        ("iRefset", "OrderedComponent") => export!(
            OrderedComponentRefsetMember,
            json::ordered_component_refset_to_json
        ),
        ("ciRefset", "OrderedAssociation") => export!(
            OrderedAssociationRefsetMember,
            json::ordered_association_refset_to_json
        ),
        ("scsRefset", "ComponentAnnotationStringValue") => {
            export!(
                ComponentAnnotationRefsetMember,
                json::component_annotation_refset_to_json
            )
        }
        ("sscsRefset", "MemberAnnotationStringValue") => {
            export!(
                MemberAnnotationRefsetMember,
                json::member_annotation_refset_to_json
            )
        }
        (_, _) => Ok(None),
    }
}

fn finish_export(
    result: Result<(), Box<dyn Error>>,
    out: String,
) -> Result<Option<String>, Box<dyn Error>> {
    match result {
        Ok(()) => Ok(Some(out)),
        Err(error) => Err(error),
    }
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

#[cfg_attr(coverage_nightly, coverage(off))]
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

    #[test]
    fn sctid_reports_extension_namespace() {
        let id = SctId::compose(
            1234,
            snomed_core::sctid::ComponentType::Concept,
            Some(1000124),
        )
        .unwrap();
        let out = run(&args(&["sctid", &id.to_string()])).unwrap();
        assert!(out.contains("long (extension)"));
        assert!(out.contains("namespace:      1000124"));
    }

    #[test]
    fn load_argument_parser_accepts_full_and_rejects_extras() {
        let full_args = args(&["release", "--full"]);
        let parsed = parse_load_args(&full_args, "load <release-dir>").unwrap();
        assert_eq!(parsed, ("release", ReleaseType::Full));

        let error = parse_load_args(&args(&["one", "two"]), "load <release-dir>").unwrap_err();
        assert!(error.to_string().contains("unexpected argument `two`"));
    }

    #[test]
    fn commands_validate_required_arguments_before_io() {
        for command in [
            "sctid", "load", "validate", "lookup", "ecl", "classify", "nnf", "export",
        ] {
            let error = run(&args(&[command])).unwrap_err();
            assert!(error.to_string().contains("usage"), "{command}: {error}");
        }

        for command in ["classify", "nnf"] {
            let error = run(&args(&[command, "--full"])).unwrap_err();
            assert!(error.to_string().contains("usage"), "{command}: {error}");
        }

        let error = run(&args(&["export", "one", "two", "three"])).unwrap_err();
        assert!(error.to_string().contains("usage: export"));
    }

    #[test]
    fn commands_propagate_io_identifier_and_rf2_failures() {
        let missing = "/definitely/not/a/snomed-release";
        for command in ["load", "validate", "classify", "nnf", "ecl"] {
            let mut command_args = vec![command.to_string(), missing.to_string()];
            if command == "ecl" {
                command_args.push("*".to_string());
            }
            assert!(run(&command_args).is_err(), "{command} hid a load failure");
        }
        assert!(run(&args(&["lookup", missing, "138875005"])).is_err());
        assert!(run(&args(&["lookup", missing, "not-an-id"])).is_err());

        let root = std::env::temp_dir().join(format!(
            "snomed-cli-unit-errors-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let root_text = root.to_str().unwrap();
        assert!(run(&args(&["classify", root_text, "not-an-id"])).is_err());
        assert!(run(&args(&["nnf", root_text, "not-an-id"])).is_err());

        let missing_file = root.join("sct2_Concept_Snapshot_INT_20240131.txt");
        assert!(run(&args(&["export", missing_file.to_str().unwrap()])).is_err());

        fs::write(&missing_file, "malformed header\n").unwrap();
        assert!(run(&args(&["export", missing_file.to_str().unwrap()])).is_err());

        fs::write(
            &missing_file,
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\nmalformed\n",
        )
        .unwrap();
        assert!(run(&args(&["export", missing_file.to_str().unwrap()])).is_err());

        fs::write(
            &missing_file,
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
             138875005\t20240131\t1\t900000000000207008\t900000000000074008\n",
        )
        .unwrap();
        let lookup = run(&args(&["lookup", root_text, "138875005"])).unwrap();
        assert!(lookup.contains("active=true"));
        assert!(!lookup.contains("FSN:"));
        let unwritable = root.join("missing-parent/out.ndjson");
        assert!(run(&args(&[
            "export",
            missing_file.to_str().unwrap(),
            unwritable.to_str().unwrap(),
        ]))
        .is_err());

        let out_file = root.join("out-file");
        fs::write(&out_file, "not a directory").unwrap();
        assert!(cmd_export_dir(&args(&[root_text, out_file.to_str().unwrap()])).is_err());

        let release = root.join("release");
        let out_dir = root.join("out-dir");
        fs::create_dir_all(&release).unwrap();
        let malformed = release.join("sct2_Concept_Snapshot_INT_20240131.txt");
        fs::write(&malformed, "malformed header\n").unwrap();
        assert!(cmd_export_dir(&args(&[
            release.to_str().unwrap(),
            out_dir.to_str().unwrap()
        ]))
        .is_err());

        fs::write(
            &malformed,
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
             138875005\t20240131\t1\t900000000000207008\t900000000000074008\n",
        )
        .unwrap();
        fs::create_dir_all(&out_dir).unwrap();
        fs::create_dir_all(out_dir.join("sct2_Concept_Snapshot_INT_20240131.ndjson")).unwrap();
        assert!(cmd_export_dir(&args(&[
            release.to_str().unwrap(),
            out_dir.to_str().unwrap()
        ]))
        .is_err());

        assert!(cmd_export_dir(&args(&[missing, out_dir.to_str().unwrap()])).is_err());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unchecked_identifier_and_non_utf8_path_helpers_are_total() {
        assert_eq!(component_type_label(SctId::new_unchecked(1)), "unknown");
        assert!(cmd_export_file(&args(&["/"])).is_err());

        #[cfg(unix)]
        {
            use std::ffi::OsString;
            use std::os::unix::ffi::OsStringExt;

            let path = std::path::PathBuf::from(OsString::from_vec(b"invalid-\xff.txt".to_vec()));
            assert!(utf8_file_name(&path).is_err());
        }
    }

    #[test]
    fn every_export_shape_propagates_typed_field_errors() {
        let root = std::env::temp_dir().join(format!(
            "snomed-cli-export-fields-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let common = "id\teffectiveTime\tactive\tmoduleId";
        let member = format!("{common}\trefsetId\treferencedComponentId");
        let cases = [
            ("sct2_Concept_Snapshot_INT_20190731.txt", format!("{common}\tdefinitionStatusId")),
            ("sct2_Description_Snapshot-en_INT_20190731.txt", format!("{common}\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId")),
            ("sct2_Relationship_Snapshot_INT_20190731.txt", format!("{common}\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId")),
            ("sct2_RelationshipConcreteValues_Snapshot_INT_20190731.txt", format!("{common}\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId")),
            ("sct2_Identifier_Snapshot_INT_20190731.txt", "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId".to_string()),
            ("der2_Refset_SimpleSnapshot_INT_20190731.txt", member.clone()),
            ("der2_cRefset_LanguageSnapshot-en_INT_20190731.txt", format!("{member}\tacceptabilityId")),
            ("der2_cRefset_AssociationSnapshot_INT_20190731.txt", format!("{member}\ttargetComponentId")),
            ("der2_cRefset_AttributeValueSnapshot_INT_20190731.txt", format!("{member}\tvalueId")),
            ("der2_sRefset_SimpleMapSnapshot_INT_20190731.txt", format!("{member}\tmapTarget")),
            ("der2_iisssccRefset_ExtendedMapSnapshot_INT_20190731.txt", format!("{member}\tmapGroup\tmapPriority\tmapRule\tmapAdvice\tmapTarget\tcorrelationId\tmapCategoryId")),
            ("der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt", format!("{member}\towlExpression")),
            ("der2_ssRefset_ModuleDependencySnapshot_INT_20190731.txt", format!("{member}\tsourceEffectiveTime\ttargetEffectiveTime")),
            ("der2_cciRefset_RefsetDescriptorSnapshot_INT_20190731.txt", format!("{member}\tattributeDescription\tattributeType\tattributeOrder")),
            ("der2_ciRefset_DescriptionTypeSnapshot_INT_20190731.txt", format!("{member}\tdescriptionFormat\tdescriptionLength")),
            ("der2_sssssssRefset_MRCMDomainSnapshot_INT_20190731.txt", format!("{member}\tdomainConstraint\tparentDomain\tproximalPrimitiveConstraint\tproximalPrimitiveRefinement\tdomainTemplateForPrecoordination\tdomainTemplateForPostcoordination\tguideURL")),
            ("der2_cissccRefset_MRCMAttributeDomainSnapshot_INT_20190731.txt", format!("{member}\tdomainId\tgrouped\tattributeCardinality\tattributeInGroupCardinality\truleStrengthId\tcontentTypeId")),
            ("der2_ssccRefset_MRCMAttributeRangeSnapshot_INT_20190731.txt", format!("{member}\trangeConstraint\tattributeRule\truleStrengthId\tcontentTypeId")),
            ("der2_cRefset_MRCMModuleScopeSnapshot_INT_20190731.txt", format!("{member}\tmrcmRuleRefsetId")),
            ("der2_iRefset_OrderedComponentSnapshot_INT_20190731.txt", format!("{member}\torder")),
            ("der2_ciRefset_OrderedAssociationSnapshot_INT_20190731.txt", format!("{member}\ttargetComponentId\torder")),
            ("der2_scsRefset_ComponentAnnotationStringValueSnapshot_INT_20190731.txt", format!("{member}\tlanguageDialectCode\ttypeId\tvalue")),
            ("der2_sscsRefset_MemberAnnotationStringValueSnapshot_INT_20190731.txt", format!("{member}\treferencedMemberId\tlanguageDialectCode\ttypeId\tvalue")),
        ];
        for (name, header) in cases {
            let valid_row = header
                .split('\t')
                .map(|column| match column {
                    "id" if name.starts_with("sct2_Concept") => "138875005".to_string(),
                    "id" if name.starts_with("sct2_Description") => {
                        SctId::compose(1001, snomed_core::sctid::ComponentType::Description, None)
                            .unwrap()
                            .to_string()
                    }
                    "id" if name.starts_with("sct2_") => {
                        SctId::compose(1001, snomed_core::sctid::ComponentType::Relationship, None)
                            .unwrap()
                            .to_string()
                    }
                    "id" => "80000000-0000-4000-8000-000000000001".to_string(),
                    "alternateIdentifier" => "CODE-1".to_string(),
                    "referencedMemberId" => "90000000-0000-4000-8000-000000000001".to_string(),
                    "effectiveTime" | "sourceEffectiveTime" | "targetEffectiveTime" => {
                        "20190731".to_string()
                    }
                    "active" | "grouped" => "1".to_string(),
                    "mapGroup" | "mapPriority" | "attributeOrder" | "descriptionLength"
                    | "order" | "relationshipGroup" => "1".to_string(),
                    "languageCode" => "en".to_string(),
                    "languageDialectCode" => "en-GB".to_string(),
                    "value" if name.contains("RelationshipConcreteValues") => "#1".to_string(),
                    "moduleId" => "900000000000207008".to_string(),
                    "definitionStatusId" => "900000000000074008".to_string(),
                    "caseSignificanceId" => "900000000000448009".to_string(),
                    "characteristicTypeId" => "900000000000011006".to_string(),
                    "modifierId" => "900000000000451002".to_string(),
                    "attributeDescription" | "attributeType" | "descriptionFormat" => {
                        "138875005".to_string()
                    }
                    column if column.ends_with("Id") => "138875005".to_string(),
                    _ => "x".to_string(),
                })
                .collect::<Vec<_>>()
                .join("\t");
            let invalid_fields = header
                .split('\t')
                .map(|_| "x")
                .collect::<Vec<_>>()
                .join("\t");
            let path = root.join(name);
            assert!(run(&args(&["export", path.to_str().unwrap()])).is_err());
            fs::write(&path, "").unwrap();
            assert!(run(&args(&["export", path.to_str().unwrap()])).is_err());
            fs::write(&path, "wrong\n").unwrap();
            assert!(run(&args(&["export", path.to_str().unwrap()])).is_err());
            fs::write(&path, format!("{header}\n{invalid_fields}\n")).unwrap();
            assert!(run(&args(&["export", path.to_str().unwrap()])).is_err());
            fs::write(&path, format!("{header}\n{valid_row}\n")).unwrap();
            assert!(!run(&args(&["export", path.to_str().unwrap()]))
                .unwrap()
                .is_empty());
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn summary_writers_handle_empty_present_and_capped_lists() {
        let id = SctId::parse("138875005").unwrap();
        let mut out = String::new();
        write_ids(&mut out, "none", &[]);
        assert!(out.is_empty());
        write_ids(&mut out, "roots", &[id]);
        assert!(out.contains("roots (1)"));

        let values = [1, 2, 3, 4, 5, 6];
        write_capped(&mut out, &values, |out, value| {
            let _ = writeln!(out, "  {value}");
        });
        assert!(out.contains("... and 1 more"));
        assert!(!out.contains("  6\n"));

        write_parse_failure(&mut out, &(id, "bad axiom".to_string()));
        assert!(out.contains("parse error on 138875005: bad axiom"));
        let skipped = [snomed_classify::SkippedConstruct::ReflexiveProperty(id); 6];
        write_capped(&mut out, &skipped, write_skipped_construct);
        assert!(out.contains("ReflexiveObjectProperty"));
        let failures = std::array::from_fn::<_, 6, _>(|_| (id, "bad axiom".to_string()));
        write_capped(&mut out, &failures, write_parse_failure);
        assert_eq!(concept_name(&SnapshotStore::builder().build(), id), "?");
    }
}
