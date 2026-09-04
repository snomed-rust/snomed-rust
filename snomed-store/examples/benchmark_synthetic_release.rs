//! Benchmarks `SnapshotStoreBuilder::load_release_dir` and `SnapshotStore`
//! queries at real-release scale, per the "Benchmark loading a real
//! International Edition snapshot" and "Decide on precomputed transitive
//! closure" tasks in `tasks.md`.
//!
//! We cannot use real SNOMED CT content here: RF2 release files are
//! licensed material (see `spec/README.md`, `.gitignore`). Instead this
//! generates a large **synthetic, fictional** RF2-shaped release —
//! structurally valid (real SCTIDs via Verhoeff, real file names, real
//! column layouts per `spec/05..08`) but with made-up terms and a random
//! hierarchy — sized to match the International Edition's ~370k active
//! concepts. It writes real RF2 files to a temp directory, loads them
//! through the same `load_release_dir` path a real release would use, and
//! times the load and a sample of hierarchy/lookup queries.
//!
//! Run with `cargo run --release --example benchmark_synthetic_release -p
//! snomed-store`. Override the concept count with the
//! `SNOMED_BENCH_CONCEPTS` environment variable.

use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use snomed_core::sctid::{ComponentType, SctId};
use snomed_rf2::release_type::ReleaseType;
use snomed_store::SnapshotStoreBuilder;

const DEFAULT_CONCEPT_COUNT: u64 = 370_000;
const DATE: &str = "20250801";

/// A minimal xorshift64* PRNG. Determinism (not cryptographic quality)
/// is what a reproducible benchmark needs.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A value in `0..bound`. `bound` must be nonzero.
    fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> std::io::Result<Self> {
        let dir = std::env::temp_dir().join(format!(
            "snomed-store-bench-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir)?;
        Ok(TempDir(dir))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn synthetic_uuid(n: u64) -> String {
    format!(
        "{:08x}-0000-4000-8000-{:012x}",
        (n >> 32) as u32,
        n & 0xFFFF_FFFF_FFFF
    )
}

struct GeneratedStats {
    concepts: u64,
    descriptions: u64,
    relationships: u64,
    language_members: u64,
    concept_ids: Vec<SctId>,
}

/// Writes a synthetic Snapshot release to `root`: a random tree of
/// `concept_count` concepts (concept 0 is the root, every other concept's
/// parent is an earlier concept, so the hierarchy is acyclic by
/// construction), each with an FSN and a preferred synonym, plus the
/// IS-A relationship to its parent.
fn generate_release(root: &Path, concept_count: u64) -> Result<GeneratedStats, Box<dyn Error>> {
    let terminology = root.join("Snapshot/Terminology");
    let language_dir = root.join("Snapshot/Refset/Language");
    fs::create_dir_all(&terminology)?;
    fs::create_dir_all(&language_dir)?;

    let mut concepts_out = BufWriter::new(File::create(
        terminology.join(format!("sct2_Concept_Snapshot_INT_{DATE}.txt")),
    )?);
    let mut descriptions_out = BufWriter::new(File::create(
        terminology.join(format!("sct2_Description_Snapshot-en_INT_{DATE}.txt")),
    )?);
    let mut relationships_out = BufWriter::new(File::create(
        terminology.join(format!("sct2_Relationship_Snapshot_INT_{DATE}.txt")),
    )?);
    let mut language_out = BufWriter::new(File::create(
        language_dir.join(format!("der2_cRefset_LanguageSnapshot-en_INT_{DATE}.txt")),
    )?);

    writeln!(
        concepts_out,
        "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId"
    )?;
    writeln!(
        descriptions_out,
        "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId"
    )?;
    writeln!(
        relationships_out,
        "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId"
    )?;
    writeln!(
        language_out,
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tacceptabilityId"
    )?;

    const CORE_MODULE: &str = "900000000000207008";
    const PRIMITIVE: &str = "900000000000074008";
    const FSN_TYPE: &str = "900000000000003001";
    const SYNONYM_TYPE: &str = "900000000000013009";
    const CASE_INSENSITIVE: &str = "900000000000448009";
    const IS_A: &str = "116680003";
    const INFERRED: &str = "900000000000011006";
    const EXISTENTIAL: &str = "900000000000451002";
    const US_ENGLISH: &str = "900000000000509007";
    const PREFERRED: &str = "900000000000548007";

    let mut rng = Rng::new(0x5eed_5eed_5eed_5eedu64);
    let mut concept_item = 100_000u64;
    let mut description_item = 100_000u64;
    let mut relationship_item = 100_000u64;
    let mut uuid_counter = 0u64;

    let mut concept_ids = Vec::with_capacity(concept_count as usize);

    for i in 0..concept_count {
        concept_item += 1;
        let concept_id = SctId::compose(concept_item, ComponentType::Concept, None)?;
        concept_ids.push(concept_id);

        writeln!(
            concepts_out,
            "{concept_id}\t{DATE}\t1\t{CORE_MODULE}\t{PRIMITIVE}"
        )?;

        description_item += 1;
        let fsn_id = SctId::compose(description_item, ComponentType::Description, None)?;
        writeln!(
            descriptions_out,
            "{fsn_id}\t{DATE}\t1\t{CORE_MODULE}\t{concept_id}\ten\t{FSN_TYPE}\tSynthetic concept {i} (finding)\t{CASE_INSENSITIVE}"
        )?;

        description_item += 1;
        let synonym_id = SctId::compose(description_item, ComponentType::Description, None)?;
        writeln!(
            descriptions_out,
            "{synonym_id}\t{DATE}\t1\t{CORE_MODULE}\t{concept_id}\ten\t{SYNONYM_TYPE}\tSynthetic concept {i} alias\t{CASE_INSENSITIVE}"
        )?;

        uuid_counter += 1;
        writeln!(
            language_out,
            "{}\t{DATE}\t1\t{CORE_MODULE}\t{US_ENGLISH}\t{synonym_id}\t{PREFERRED}",
            synthetic_uuid(uuid_counter)
        )?;

        if i > 0 {
            let parent_index = rng.below(i);
            let parent_id = concept_ids[parent_index as usize];
            relationship_item += 1;
            let rel_id = SctId::compose(relationship_item, ComponentType::Relationship, None)?;
            writeln!(
                relationships_out,
                "{rel_id}\t{DATE}\t1\t{CORE_MODULE}\t{concept_id}\t{parent_id}\t0\t{IS_A}\t{INFERRED}\t{EXISTENTIAL}"
            )?;
        }
    }

    concepts_out.flush()?;
    descriptions_out.flush()?;
    relationships_out.flush()?;
    language_out.flush()?;

    Ok(GeneratedStats {
        concepts: concept_count,
        descriptions: concept_count * 2,
        relationships: concept_count.saturating_sub(1),
        language_members: concept_count,
        concept_ids,
    })
}

fn time_query<F: FnMut() -> usize>(label: &str, sample_len: usize, mut f: F) -> (Duration, usize) {
    let start = Instant::now();
    let mut total = 0usize;
    for _ in 0..sample_len {
        total += f();
    }
    let elapsed = start.elapsed();
    println!(
        "  {label:<28} {sample_len:>6} calls in {elapsed:>10.2?}  ({:>8.2?} avg, {total} total results)",
        elapsed / sample_len as u32,
    );
    (elapsed, total)
}

fn main() -> Result<(), Box<dyn Error>> {
    let concept_count: u64 = std::env::var("SNOMED_BENCH_CONCEPTS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CONCEPT_COUNT);

    println!(
        "Synthetic release benchmark: {concept_count} concepts (fictional data, structurally \
         RF2-shaped; no real SNOMED CT content is used)"
    );

    let tmp = TempDir::new()?;

    let gen_start = Instant::now();
    let stats = generate_release(tmp.path(), concept_count)?;
    println!(
        "Generated + wrote to disk: {} concepts, {} descriptions, {} relationships, {} language \
         members in {:.2?}",
        stats.concepts,
        stats.descriptions,
        stats.relationships,
        stats.language_members,
        gen_start.elapsed()
    );

    let mut builder = SnapshotStoreBuilder::new();
    let load_start = Instant::now();
    let report = builder.load_release_dir(tmp.path(), ReleaseType::Snapshot)?;
    let load_elapsed = load_start.elapsed();
    let total_rows =
        stats.concepts + stats.descriptions + stats.relationships + stats.language_members;
    println!(
        "load_release_dir: {} files loaded, {} skipped, in {:.2?} ({:.0} rows/sec)",
        report.loaded.len(),
        report.skipped.len(),
        load_elapsed,
        total_rows as f64 / load_elapsed.as_secs_f64()
    );

    let build_start = Instant::now();
    let store = builder.build();
    println!(
        "SnapshotStoreBuilder::build() (derived indexes): {:.2?}",
        build_start.elapsed()
    );
    println!(
        "Store: {} concepts ({} active)",
        store.concept_count(),
        store.active_concepts().count()
    );

    let mut rng = Rng::new(0xdeca_deca_deca_decau64);
    let sample_len = (stats.concept_ids.len()).min(2000);
    let sample: Vec<SctId> = (0..sample_len)
        .map(|_| stats.concept_ids[rng.below(stats.concept_ids.len() as u64) as usize])
        .collect();
    let root_id = stats.concept_ids[0];

    println!("\nQuery benchmarks ({sample_len} random concepts):");
    let mut sample_iter = sample.iter();
    time_query("ancestors()", sample_len, || {
        store.ancestors(*sample_iter.next().unwrap()).len()
    });
    let mut sample_iter = sample.iter();
    time_query("descendants()", sample_len, || {
        store.descendants(*sample_iter.next().unwrap()).len()
    });
    let mut sample_iter = sample.iter();
    time_query("subsumes(root, x)", sample_len, || {
        store.subsumes(root_id, *sample_iter.next().unwrap()) as usize
    });
    let mut sample_iter = sample.iter();
    time_query("is_active()", sample_len, || {
        store.is_active(*sample_iter.next().unwrap()) as usize
    });
    let mut sample_iter = sample.iter();
    time_query("fsn()", sample_len, || {
        store.fsn(*sample_iter.next().unwrap()).is_some() as usize
    });
    let mut sample_iter = sample.iter();
    time_query("preferred_term()", sample_len, || {
        store
            .preferred_term(
                *sample_iter.next().unwrap(),
                snomed_core::constants::US_ENGLISH_LANGUAGE_REFSET,
            )
            .is_some() as usize
    });

    Ok(())
}
