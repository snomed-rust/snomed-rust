//! ECL benchmarks (`spec/10-ecl.md`): parsing is per-query and cheap;
//! evaluation walks the hierarchy and is what a terminology server pays for
//! on every request.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use snomed_benches::synthetic_store;
use snomed_ecl::{evaluate, parse};

const CONCEPTS: u64 = 20_000;

/// Representative constraints: a bare concept, each hierarchy operator, a
/// refinement, a conjunction, an exclusion, and `^ *`.
///
/// `member_of_wildcard` is here rather than in a group of its own because
/// it needs no fixture change: the synthetic release's Language refset
/// members are exactly the "every refset in the substrate" union `^ *`
/// walks (spec/10 rule 16), so it measures real work on the existing
/// generator — and the existing baselines stay comparable
/// (`spec/rust-bench.md` rule 3).
fn expressions(root: &str, mid: &str, member: &str) -> Vec<(&'static str, String)> {
    vec![
        ("self", root.to_string()),
        ("member_of_wildcard", "^ *".to_string()),
        ("refset_containing_any", format!("^R {member}")),
        ("refset_containing_any_set", format!("^R (<< {root})")),
        ("descendant_or_self", format!("<< {root}")),
        ("descendant", format!("< {root}")),
        ("ancestor_or_self", format!(">> {mid}")),
        ("ancestor", format!("> {mid}")),
        ("conjunction", format!("<< {root} AND << {mid}")),
        ("exclusion", format!("<< {root} MINUS << {mid}")),
        ("disjunction", format!("<< {mid} OR << {root}")),
    ]
}

/// Description filter constraints, whose cost profile differs from the
/// hierarchy operators above: they visit every candidate's descriptions
/// and do string work per description, so they scale with description
/// count rather than hierarchy depth (`spec/10-ecl-filters.md`).
fn filter_expressions(root: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "term_match",
            format!("<< {root} {{{{ D term = \"synthetic\" }}}}"),
        ),
        (
            "term_match_two_words",
            format!("<< {root} {{{{ D term = \"synthetic concept\" }}}}"),
        ),
        (
            "term_wild",
            format!("<< {root} {{{{ D term = wild:\"*alias\" }}}}"),
        ),
        (
            "term_exact",
            format!("<< {root} {{{{ D term = exact:\"Synthetic concept 1 alias\" }}}}"),
        ),
        ("type_token", format!("<< {root} {{{{ D type = fsn }}}}")),
        ("language", format!("<< {root} {{{{ D language = en }}}}")),
        (
            "dialect_preferred",
            format!(
                "<< {root} {{{{ D dialectId = {} (preferred) }}}}",
                snomed_core::constants::US_ENGLISH_LANGUAGE_REFSET
            ),
        ),
        (
            "conjunction",
            format!("<< {root} {{{{ D type = syn, term = \"alias\", language = en }}}}"),
        ),
        // The expression-valued kinds: their value is evaluated once per
        // query, and these guard that it stays that way (spec/10 rule 0).
        (
            "type_id_expression",
            format!("<< {root} {{{{ D typeId = (900000000000003001 OR 900000000000013009) }}}}"),
        ),
        (
            "module_id_expression",
            format!("<< {root} {{{{ D moduleId = << 900000000000207008 }}}}"),
        ),
    ]
}

/// Refinements, whose cost the description filters' benchmark doesn't
/// cover: an attribute constraint evaluates its attribute-name and value
/// expressions once per query, and matches every candidate's
/// relationships against them (`spec/10-ecl.md`).
fn refinement_expressions(root: &str, attr: &str, value: &str) -> Vec<(&'static str, String)> {
    vec![
        ("attribute", format!("<< {root} : {attr} = {value}")),
        (
            "attribute_hierarchy_value",
            format!("<< {root} : {attr} = << {value}"),
        ),
        ("negated", format!("<< {root} : {attr} != {value}")),
        (
            "conjunction",
            format!("<< {root} : {attr} = {value} AND {attr} = << {value}"),
        ),
        ("group", format!("<< {root} : {{ {attr} = {value} }}")),
        (
            "nested_value",
            format!("<< {root} : {attr} = (<< {value} : {attr} = *)"),
        ),
    ]
}

/// `dottedExpressionConstraint` (spec/10 rule 15), and the reverse-flag
/// refinement it is sugar for, over the same relationships — so the two
/// implementations of one semantics can be compared directly rather than
/// assumed to cost the same.
fn dotted_expressions(root: &str, attr: &str) -> Vec<(&'static str, String)> {
    vec![
        ("dotted", format!("<< {root} . {attr}")),
        ("dotted_chain", format!("<< {root} . {attr} . {attr}")),
        ("dotted_wildcard_attribute", format!("<< {root} . *")),
        (
            "reverse_flag_equivalent",
            format!("* : R {attr} = << {root}"),
        ),
    ]
}

fn bench_ecl(c: &mut Criterion) {
    let (store, ids) = synthetic_store(CONCEPTS);
    let root = ids[0].to_string();
    let mid = ids[ids.len() / 2].to_string();
    // Divisible by 21, so it is in both Simple refsets the generator
    // emits (`synthetic_simple_refsets`, membership at i % 3 and i % 7) —
    // otherwise `^R` would look up a concept in no refset at all.
    let member = ids[(ids.len() / 2 / 21) * 21].to_string();
    let cases = expressions(&root, &mid, &member);

    let mut group = c.benchmark_group("ecl_parse");
    for (name, text) in &cases {
        group.bench_function(*name, |b| b.iter(|| black_box(parse(black_box(text)))));
    }
    group.finish();

    let mut group = c.benchmark_group("ecl_evaluate");
    group.sample_size(30);
    for (name, text) in &cases {
        let expr = parse(text).expect("benchmark expression parses");
        // spec/rust-bench.md rule 2. `^R` in particular reads a
        // concept-only index that stays empty unless the generator emits
        // Simple refset members, and an empty lookup times fast and looks
        // fine.
        assert!(
            !evaluate(&expr, &store).is_empty(),
            "`{text}` matches nothing, so `{name}` would measure an empty result"
        );
        group.bench_function(*name, |b| {
            b.iter(|| black_box(evaluate(black_box(&expr), black_box(&store)).len()))
        });
    }
    group.finish();

    // Name the attribute type the release actually uses, so these
    // measure attribute matching rather than its absence.
    let attribute = snomed_benches::synthetic_attribute_type().to_string();
    let refinements = refinement_expressions(&root, &attribute, &root);
    let mut group = c.benchmark_group("ecl_refinements");
    group.sample_size(20);
    for (name, text) in &refinements {
        let expr = parse(text).expect("benchmark refinement parses");
        group.bench_function(*name, |b| {
            b.iter(|| black_box(evaluate(black_box(&expr), black_box(&store)).len()))
        });
    }
    group.finish();

    let dotted = dotted_expressions(&root, &attribute);
    let mut group = c.benchmark_group("ecl_dotted");
    group.sample_size(20);
    for (name, text) in &dotted {
        let expr = parse(text).expect("benchmark dotted expression parses");
        // spec/rust-bench.md rule 2: a dotted expression over a release
        // with no non-IS-A relationships would evaluate to the empty set
        // and time nothing. The generator emits attribute relationships
        // (`synthetic_attribute_type`); this makes that a hard
        // precondition rather than an assumption.
        assert!(
            !evaluate(&expr, &store).is_empty(),
            "`{text}` matches nothing, so `{name}` would measure an empty traversal"
        );
        group.bench_function(*name, |b| {
            b.iter(|| black_box(evaluate(black_box(&expr), black_box(&store)).len()))
        });
    }
    group.finish();

    let filters = filter_expressions(&root);
    let mut group = c.benchmark_group("ecl_description_filters");
    group.sample_size(20);
    for (name, text) in &filters {
        let expr = parse(text).expect("benchmark filter parses");
        group.bench_function(*name, |b| {
            b.iter(|| black_box(evaluate(black_box(&expr), black_box(&store)).len()))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ecl);
criterion_main!(benches);
