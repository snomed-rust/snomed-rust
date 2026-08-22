//! `ValueSet/$expand` (Value Set Expansion), per `spec/11-fhir.md`.

use std::collections::HashSet;

use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::error::FhirError;
use crate::lookup::{designations_for, display_for};
use crate::{Designation, SNOMED_CT_SYSTEM};

/// One of SNOMED CT's implicit value set forms
/// ([hl7.org/fhir/R4/snomedct.html](https://www.hl7.org/fhir/R4/snomedct.html)),
/// parsed from a `$expand` `url`. Exposed publicly since knowing which
/// form a `url` parsed to is independently useful to a hosting server
/// (e.g. for logging or caching), not just an internal detail of
/// [`expand`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImplicitValueSet {
    /// `?fhir_vs` — every concept.
    AllConcepts,
    /// `?fhir_vs=isa/[sctid]` — the concept and its descendants.
    IsA(SctId),
    /// `?fhir_vs=refset/[sctid]` — members of the named refset.
    Refset(SctId),
    /// `?fhir_vs=ecl/[ecl]` — an arbitrary ECL expression, evaluated via
    /// `snomed-ecl`. Holds the raw (already percent-decoded — this crate
    /// does no URL decoding of its own, see [`expand`]'s docs) ECL text.
    Ecl(String),
    /// `?fhir_vs=refset` (bare, no `[sctid]`) — every concept that is
    /// itself a refset identifier with at least one active member.
    AllRefsets,
}

/// Options for [`expand`], mirroring `$expand`'s optional input
/// parameters this crate supports (spec/11).
#[derive(Debug, Clone, Copy, Default)]
pub struct ExpandOptions<'a> {
    /// `activeOnly` — exclude inactive concepts when `true`.
    pub active_only: bool,
    /// `includeDesignations` — populate each `contains` entry's
    /// `designation` list when `true`.
    pub include_designations: bool,
    /// Stands in for `displayLanguage`/designation language, same
    /// rationale as `lookup`'s `language_refset` (spec/11's "Dialect
    /// instead of `displayLanguage`" note).
    pub language_refset: Option<SctId>,
    /// `filter` — case-insensitive substring match against each
    /// candidate's active description terms. A deliberate simplification
    /// versus FHIR's "server decides, ideally with relevance ranking"
    /// latitude (spec/11).
    pub filter: Option<&'a str>,
    /// `count` — maximum number of `contains` entries to return. `None`
    /// returns every match after `offset`.
    pub count: Option<usize>,
    /// `offset` — how many matches (after filtering) to skip.
    pub offset: usize,
}

/// One `ValueSet.expansion.contains` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpansionContains {
    pub system: &'static str,
    pub code: SctId,
    /// Same fallback rule as `lookup`'s `display` (spec/11): preferred
    /// term in `language_refset` if given and found, else the FSN, else
    /// `None` for a description-less concept.
    pub display: Option<String>,
    /// Populated only when `ExpandOptions::include_designations` is set;
    /// otherwise always empty.
    pub designation: Vec<Designation>,
}

/// The result of [`expand`]. Not a FHIR `ValueSet` resource — a hosting
/// server builds `ValueSet.expansion` from these fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    /// Total matches after `activeOnly`/`filter`, before `offset`/`count`
    /// pagination — i.e. how many there would be with no paging at all.
    pub total: usize,
    /// The `offset` actually applied (clamped to `0..=total`).
    pub offset: usize,
    pub contains: Vec<ExpansionContains>,
}

/// Parses a `$expand` `url` into the [`ImplicitValueSet`] form it
/// describes, per spec/11's five-row table — all five are recognized.
/// Returns [`FhirError::UnsupportedSystem`] when the URL's base isn't
/// [`SNOMED_CT_SYSTEM`], and [`FhirError::UnsupportedValueSet`] for any
/// query shape that isn't one of the five (or an `isa/`/`refset/` id
/// that isn't a syntactically valid SCTID).
///
/// **No percent-decoding**: this crate has no URL/percent-decoding
/// parser (zero external dependencies) and doesn't add one just for this.
/// `url`'s query portion — in particular the ECL text after `ecl/` — must
/// already be decoded by the caller (e.g. spaces as spaces, not `%20`).
pub fn parse_implicit_value_set(url: &str) -> Result<ImplicitValueSet, FhirError> {
    let Some((base, query)) = url.split_once('?') else {
        return Err(FhirError::UnsupportedValueSet(url.to_string()));
    };
    if base != SNOMED_CT_SYSTEM {
        return Err(FhirError::UnsupportedSystem(base.to_string()));
    }

    if query == "fhir_vs" {
        return Ok(ImplicitValueSet::AllConcepts);
    }
    let Some(value) = query.strip_prefix("fhir_vs=") else {
        return Err(FhirError::UnsupportedValueSet(url.to_string()));
    };
    // A URL is percent-encoded by definition (RFC 3986), and the forms
    // that carry ECL are unusable without decoding: FHIR's own published
    // example is `?fhir_vs=ecl/%3C%3C%2027624003`. Decoding happens after
    // the `?` and `fhir_vs=` splits, so an encoded `%3F`/`%3D` in the
    // payload can never be mistaken for a real delimiter (spec/11).
    let value = &percent_decode(value, url)?;
    if value == "refset" {
        return Ok(ImplicitValueSet::AllRefsets);
    }
    if let Some(id) = value.strip_prefix("isa/") {
        return SctId::parse(id)
            .map(ImplicitValueSet::IsA)
            .map_err(|_| FhirError::UnsupportedValueSet(url.to_string()));
    }
    if let Some(id) = value.strip_prefix("refset/") {
        return SctId::parse(id)
            .map(ImplicitValueSet::Refset)
            .map_err(|_| FhirError::UnsupportedValueSet(url.to_string()));
    }
    if let Some(ecl) = value.strip_prefix("ecl/") {
        return Ok(ImplicitValueSet::Ecl(ecl.to_string()));
    }
    Err(FhirError::UnsupportedValueSet(url.to_string()))
}

/// Expands the implicit value set named by `url`, per spec/11's
/// `$expand` section. See [`parse_implicit_value_set`] for the URL forms
/// understood and [`ExpandOptions`] for the supported optional
/// parameters.
pub fn expand(
    store: &SnapshotStore,
    url: &str,
    options: &ExpandOptions,
) -> Result<Expansion, FhirError> {
    let value_set = parse_implicit_value_set(url)?;
    let ids = evaluate(store, &value_set)?;

    let mut sorted: Vec<SctId> = ids.into_iter().collect();
    sorted.sort();

    let filtered: Vec<SctId> = sorted
        .into_iter()
        .filter(|&id| !options.active_only || store.is_active(id))
        .filter(|&id| match options.filter {
            Some(text) => matches_filter(store, id, text),
            None => true,
        })
        .collect();

    let total = filtered.len();
    let offset = options.offset.min(total);
    let end = match options.count {
        Some(count) => offset.saturating_add(count).min(total),
        None => total,
    };

    let contains = filtered[offset..end]
        .iter()
        .map(|&code| ExpansionContains {
            system: SNOMED_CT_SYSTEM,
            code,
            display: display_for(store, options.language_refset, code),
            designation: if options.include_designations {
                designations_for(store, options.language_refset, code)
            } else {
                Vec::new()
            },
        })
        .collect();

    Ok(Expansion {
        total,
        offset,
        contains,
    })
}

/// Decodes `%XX` escapes in one query-parameter value.
///
/// `+` is left alone: that spelling of a space belongs to
/// `application/x-www-form-urlencoded` form bodies, not to URI query
/// syntax (RFC 3986), and ECL uses `+` in its own right — decoding it
/// would corrupt a concrete numeric value like `#+5`.
fn percent_decode(value: &str, url: &str) -> Result<String, FhirError> {
    if !value.contains('%') {
        return Ok(value.to_string());
    }
    let malformed = || FhirError::MalformedUrlEncoding(url.to_string());
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'%' {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        let hex = bytes.get(i + 1..i + 3).ok_or_else(malformed)?;
        let digit = |b: u8| (b as char).to_digit(16).ok_or_else(malformed);
        out.push((digit(hex[0])? * 16 + digit(hex[1])?) as u8);
        i += 3;
    }
    String::from_utf8(out).map_err(|_| malformed())
}

fn evaluate(
    store: &SnapshotStore,
    value_set: &ImplicitValueSet,
) -> Result<HashSet<SctId>, FhirError> {
    Ok(match value_set {
        ImplicitValueSet::AllConcepts => store.concepts().map(|c| c.id).collect(),
        ImplicitValueSet::IsA(id) => {
            // Reflexive: `id` itself only if it's actually a known
            // concept — mirrors snomed-ecl's `<<` (DescendantOrSelfOf)
            // exactly (spec/10, spec/11 rule 5: reuse SnapshotStore's
            // existing hierarchy primitives, not a fresh traversal).
            let mut result = store.descendants(*id);
            if store.concept(*id).is_some() {
                result.insert(*id);
            }
            result
        }
        ImplicitValueSet::Refset(id) => store.refset_members(*id).collect(),
        ImplicitValueSet::Ecl(text) => {
            let expr = snomed_ecl::parse(text).map_err(|e| FhirError::InvalidEcl(e.to_string()))?;
            snomed_ecl::evaluate(&expr, store)
        }
        ImplicitValueSet::AllRefsets => store.refset_ids().collect(),
    })
}

fn matches_filter(store: &SnapshotStore, id: SctId, filter: &str) -> bool {
    let needle = filter.to_lowercase();
    store
        .descriptions_of(id)
        .any(|d| d.active && d.term.to_lowercase().contains(&needle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::{Concept, Description, Relationship};
    use snomed_core::constants;
    use snomed_core::member_id::MemberId;
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;

    const ROOT: SctId = constants::ROOT_CONCEPT;
    const FINDING: SctId = SctId::new_unchecked(404684003);
    const DISEASE: SctId = SctId::new_unchecked(64572001);

    fn concept(id: SctId, active: bool) -> Concept {
        Concept {
            id,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    fn fsn(id: u64, concept_id: SctId, term: &str) -> Description {
        Description {
            id: SctId::compose(id, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: term.to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        }
    }

    fn is_a(item: u64, source: SctId, destination: SctId) -> Relationship {
        Relationship {
            id: SctId::compose(1000 + item, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: source,
            destination_id: destination,
            relationship_group: 0,
            type_id: constants::IS_A,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        }
    }

    // ROOT <- FINDING <- DISEASE, all active with FSNs, DISEASE inactive
    // in one variant used by the activeOnly test.
    fn fixture(disease_active: bool) -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT, true));
        b.add_concept(concept(FINDING, true));
        b.add_concept(concept(DISEASE, disease_active));
        b.add_description(fsn(2001, ROOT, "SNOMED CT Concept (SNOMED RT+CTV3)"));
        b.add_description(fsn(2002, FINDING, "Clinical finding (finding)"));
        b.add_description(fsn(2003, DISEASE, "Disease (disorder)"));
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.build()
    }

    #[test]
    fn parses_all_five_documented_url_forms() {
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs").unwrap(),
            ImplicitValueSet::AllConcepts
        );
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=isa/404684003").unwrap(),
            ImplicitValueSet::IsA(FINDING)
        );
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=refset/404684003").unwrap(),
            ImplicitValueSet::Refset(FINDING)
        );
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=ecl/<< 404684003").unwrap(),
            ImplicitValueSet::Ecl("<< 404684003".to_string())
        );
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=refset").unwrap(),
            ImplicitValueSet::AllRefsets
        );
    }

    #[test]
    fn rejects_a_genuinely_unrecognized_query_shape() {
        let err = parse_implicit_value_set("http://snomed.info/sct?fhir_vs=nonsense").unwrap_err();
        assert!(matches!(err, FhirError::UnsupportedValueSet(_)));
    }

    #[test]
    fn rejects_a_non_snomed_system_url() {
        let err = parse_implicit_value_set("http://loinc.org?fhir_vs").unwrap_err();
        assert!(matches!(err, FhirError::UnsupportedSystem(s) if s == "http://loinc.org"));
    }

    #[test]
    fn rejects_an_unrecognized_url() {
        let err = parse_implicit_value_set("not a url at all").unwrap_err();
        assert!(matches!(err, FhirError::UnsupportedValueSet(_)));
    }

    #[test]
    fn all_concepts_expands_every_concept_sorted() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs",
            &ExpandOptions::default(),
        )
        .unwrap();
        assert_eq!(result.total, 3);
        let codes: Vec<SctId> = result.contains.iter().map(|c| c.code).collect();
        assert_eq!(codes, {
            let mut v = vec![ROOT, FINDING, DISEASE];
            v.sort();
            v
        });
    }

    #[test]
    fn isa_expands_self_and_descendants() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs=isa/404684003",
            &ExpandOptions::default(),
        )
        .unwrap();
        let codes: Vec<SctId> = result.contains.iter().map(|c| c.code).collect();
        let mut expected = vec![FINDING, DISEASE];
        expected.sort();
        assert_eq!(codes, expected);
    }

    #[test]
    fn isa_of_an_unknown_concept_is_empty_not_an_error() {
        let store = fixture(true);
        let unknown = SctId::compose(9999, ComponentType::Concept, None).unwrap();
        let result = expand(
            &store,
            &format!("http://snomed.info/sct?fhir_vs=isa/{unknown}"),
            &ExpandOptions::default(),
        )
        .unwrap();
        assert_eq!(result.total, 0);
    }

    #[test]
    fn ecl_form_evaluates_via_snomed_ecl() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs=ecl/<< 404684003 MINUS << 64572001",
            &ExpandOptions::default(),
        )
        .unwrap();
        let codes: Vec<SctId> = result.contains.iter().map(|c| c.code).collect();
        assert_eq!(codes, vec![FINDING]);
    }

    #[test]
    fn invalid_ecl_is_reported_not_panicked() {
        let store = fixture(true);
        let err = expand(
            &store,
            "http://snomed.info/sct?fhir_vs=ecl/<<<",
            &ExpandOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(err, FhirError::InvalidEcl(_)));
    }

    #[test]
    fn active_only_excludes_inactive_concepts() {
        let store = fixture(false); // DISEASE inactive
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs",
            &ExpandOptions {
                active_only: true,
                ..Default::default()
            },
        )
        .unwrap();
        let codes: Vec<SctId> = result.contains.iter().map(|c| c.code).collect();
        assert!(!codes.contains(&DISEASE), "{codes:?}");
        assert_eq!(result.total, 2);
    }

    #[test]
    fn filter_matches_description_text_case_insensitively() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs",
            &ExpandOptions {
                filter: Some("disease"),
                ..Default::default()
            },
        )
        .unwrap();
        let codes: Vec<SctId> = result.contains.iter().map(|c| c.code).collect();
        assert_eq!(codes, vec![DISEASE]);
    }

    #[test]
    fn count_and_offset_paginate_after_total_is_computed() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs",
            &ExpandOptions {
                count: Some(1),
                offset: 1,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result.total, 3, "total reflects all matches, not the page");
        assert_eq!(result.offset, 1);
        assert_eq!(result.contains.len(), 1);
    }

    #[test]
    fn display_and_designations_follow_lookup_semantics() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs=isa/404684003",
            &ExpandOptions {
                include_designations: true,
                ..Default::default()
            },
        )
        .unwrap();
        let finding = result.contains.iter().find(|c| c.code == FINDING).unwrap();
        assert_eq!(
            finding.display.as_deref(),
            Some("Clinical finding (finding)")
        );
        assert_eq!(finding.designation.len(), 1);
    }

    #[test]
    fn designations_are_empty_when_not_requested() {
        let store = fixture(true);
        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs=isa/404684003",
            &ExpandOptions::default(),
        )
        .unwrap();
        assert!(result.contains.iter().all(|c| c.designation.is_empty()));
    }

    #[test]
    fn bare_refset_form_expands_every_refset_identifier() {
        use snomed_rf2::refset::{RefsetMemberCore, SimpleRefsetMember};

        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING, true));
        b.add_concept(concept(DISEASE, true));
        // DISEASE is used as a refset identifier (has one active member);
        // FINDING is a plain concept, never a refset identifier.
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000031").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: DISEASE,
                referenced_component_id: FINDING,
            },
        });
        let store = b.build();

        let result = expand(
            &store,
            "http://snomed.info/sct?fhir_vs=refset",
            &ExpandOptions::default(),
        )
        .unwrap();
        let codes: Vec<SctId> = result.contains.iter().map(|c| c.code).collect();
        assert_eq!(codes, vec![DISEASE]);
    }

    #[test]
    fn percent_encoded_urls_decode() {
        // FHIR's own published example spelling of the ECL form, from the
        // R4 SNOMED CT page: the operators and spaces arrive encoded.
        let expected = ImplicitValueSet::Ecl("<< 27624003".to_string());
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=ecl/%3C%3C%2027624003"),
            Ok(expected.clone())
        );
        // Lowercase hex digits are equally valid (RFC 3986).
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=ecl/%3c%3c%2027624003"),
            Ok(expected)
        );
        // An already-decoded url still works — decoding is idempotent for
        // payloads that contain no `%`.
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=ecl/<< 27624003"),
            Ok(ImplicitValueSet::Ecl("<< 27624003".to_string()))
        );
        // Non-ECL forms decode the same way.
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=isa%2f138875005"),
            Ok(ImplicitValueSet::IsA(constants::ROOT_CONCEPT))
        );
    }

    #[test]
    fn plus_is_a_literal_plus_not_a_space() {
        // `+` for space is form-encoding, not URI syntax — and ECL uses
        // `+` in concrete numeric values, so decoding it would corrupt
        // the expression rather than merely alter it.
        assert_eq!(
            parse_implicit_value_set("http://snomed.info/sct?fhir_vs=ecl/*:1142135004=%23+5"),
            Ok(ImplicitValueSet::Ecl("*:1142135004=#+5".to_string()))
        );
    }

    #[test]
    fn malformed_percent_encoding_is_its_own_error() {
        // A truncated or non-hex escape is a malformed *url*, which a
        // hosting server reports differently from an unsupported value set.
        for url in [
            "http://snomed.info/sct?fhir_vs=ecl/%",
            "http://snomed.info/sct?fhir_vs=ecl/%3",
            "http://snomed.info/sct?fhir_vs=ecl/%zz",
            "http://snomed.info/sct?fhir_vs=ecl/%ff%fe",
        ] {
            assert_eq!(
                parse_implicit_value_set(url),
                Err(FhirError::MalformedUrlEncoding(url.to_string())),
                "{url} should be rejected as malformed percent-encoding"
            );
        }
    }
}
