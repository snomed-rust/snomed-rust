//! `CodeSystem/$lookup` (Concept Look Up & Decomposition), per
//! `spec/11-fhir.md`.

use snomed_classify::{NecessaryNormalForm, NecessaryNormalFormReport};
use snomed_core::constants;
use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::error::FhirError;
use crate::normal_form;
use crate::SNOMED_CT_SYSTEM;

/// The `$lookup` output, mapped onto what a `SnapshotStore` can answer
/// (spec/11's `$lookup` table). Not a FHIR `Parameters` resource — a
/// hosting server builds that from these fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupResult {
    /// Display identifier for the code system: always `"SNOMED CT"`.
    pub name: &'static str,
    /// Passed through from the caller-supplied version, if given — a
    /// `SnapshotStore` doesn't track its own release provenance.
    pub version: Option<String>,
    /// The preferred term in `language_refset` if one was given and
    /// found, else the active FSN, else `None` if this concept has no
    /// locatable description at all (a real code, just description-less
    /// data — distinct from `FhirError::UnknownCode`, which means the
    /// code itself doesn't resolve to a concept).
    pub display: Option<String>,
    /// The active `TextDefinition` description term, if one is loaded
    /// (spec/06) — SNOMED CT text definitions are optional, sparse
    /// licensed content.
    pub definition: Option<String>,
    /// Every active FSN/synonym description, each with its
    /// preferred/acceptable/unspecified use in `language_refset`.
    /// `TextDefinition` rows are excluded here — they're already covered
    /// by `definition`.
    pub designation: Vec<Designation>,
    /// The requested (or, if none were requested, the default) set of
    /// properties this crate can compute.
    pub property: Vec<LookupProperty>,
}

/// One `$lookup` `designation` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Designation {
    pub language_code: String,
    pub value: String,
    pub use_: DesignationUse,
}

/// Where a designation's acceptability came from `language_refset`, per
/// the RF2 Language reference set's `acceptabilityId` (spec/08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignationUse {
    Preferred,
    Acceptable,
    /// No `language_refset` was given, or this description isn't an
    /// active member of it.
    Unspecified,
}

/// A `$lookup` `property` this crate can compute, with its value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LookupProperty {
    Inactive(bool),
    ModuleId(SctId),
    SufficientlyDefined(bool),
    /// SNOMED CT Compositional Grammar text for the concept's necessary
    /// normal form, with `|term|` labels (spec/11, `crate::normal_form`).
    /// Only returned when `lookup`'s `nnf_report` parameter is `Some`.
    NormalForm(String),
    /// Same as `NormalForm`, without `|term|` labels or whitespace.
    NormalFormTerse(String),
}

impl LookupProperty {
    /// The FHIR property `code` this value would be reported under.
    pub fn code(&self) -> &'static str {
        match self {
            LookupProperty::Inactive(_) => "inactive",
            LookupProperty::ModuleId(_) => "moduleId",
            LookupProperty::SufficientlyDefined(_) => "sufficientlyDefined",
            LookupProperty::NormalForm(_) => "normalForm",
            LookupProperty::NormalFormTerse(_) => "normalFormTerse",
        }
    }
}

const DEFAULT_PROPERTIES: [&str; 3] = ["inactive", "moduleId", "sufficientlyDefined"];

/// Looks up `code`, per spec/11's `$lookup` section.
///
/// `language_refset` stands in for FHIR's `displayLanguage` (spec/11's
/// "Dialect instead of `displayLanguage`" note — SNOMED CT has no BCP-47
/// tags, only language refset ids). `properties` names which `property`
/// values to return; an empty slice returns this crate's default set
/// (`inactive`, `moduleId`, `sufficientlyDefined`) rather than nothing,
/// mirroring "if no properties are specified, the server chooses what to
/// return". Any requested name this crate can't compute is rejected with
/// [`FhirError::UnsupportedProperty`] (spec/11 rule 4) rather than
/// silently omitted.
///
/// `nnf_report` supplies `normalForm`/`normalFormTerse`: a
/// `NecessaryNormalFormReport` computed *once*, by the caller, over the
/// whole release's OWL axioms (`snomed_classify::necessary_normal_form`)
/// — `lookup` never computes classification itself, since doing so per
/// call would mean re-running full DL saturation on every request (see
/// `AGENTS/fhir-engineer.md`). Requesting either property with
/// `nnf_report: None` is rejected with
/// [`FhirError::MissingClassification`], distinct from
/// `UnsupportedProperty`: the property *is* supported, this call just
/// didn't supply what it needs. A concept absent from
/// `nnf_report.forms` (never named in the classified axioms) renders as
/// an empty expression, not an error — mirroring `display: None` for a
/// description-less concept: a legitimate absence of data, not a
/// lookup failure.
pub fn lookup(
    store: &SnapshotStore,
    system: &str,
    code: SctId,
    version: Option<&str>,
    language_refset: Option<SctId>,
    properties: &[&str],
    nnf_report: Option<&NecessaryNormalFormReport>,
) -> Result<LookupResult, FhirError> {
    if system != SNOMED_CT_SYSTEM {
        return Err(FhirError::UnsupportedSystem(system.to_string()));
    }
    let concept = store.concept(code).ok_or(FhirError::UnknownCode(code))?;

    let display = display_for(store, language_refset, code);

    let definition = store
        .descriptions_of(code)
        .find(|d| d.active && d.type_id == constants::TEXT_DEFINITION)
        .map(|d| d.term.clone());

    let designation = designations_for(store, language_refset, code);

    let requested: &[&str] = if properties.is_empty() {
        &DEFAULT_PROPERTIES
    } else {
        properties
    };
    let mut property = Vec::with_capacity(requested.len());
    for &name in requested {
        property.push(match name {
            "inactive" => LookupProperty::Inactive(!concept.active),
            "moduleId" => LookupProperty::ModuleId(concept.module_id),
            "sufficientlyDefined" => {
                LookupProperty::SufficientlyDefined(concept.is_sufficiently_defined())
            }
            "normalForm" => LookupProperty::NormalForm(normal_form_property(
                store,
                language_refset,
                code,
                nnf_report,
                name,
                false,
            )?),
            "normalFormTerse" => LookupProperty::NormalFormTerse(normal_form_property(
                store,
                language_refset,
                code,
                nnf_report,
                name,
                true,
            )?),
            other => return Err(FhirError::UnsupportedProperty(other.to_string())),
        });
    }

    Ok(LookupResult {
        name: "SNOMED CT",
        version: version.map(str::to_string),
        display,
        definition,
        designation,
        property,
    })
}

/// Renders `code`'s necessary normal form as compositional grammar text
/// (`crate::normal_form`), or rejects with
/// [`FhirError::MissingClassification`] if `nnf_report` wasn't supplied.
/// A concept `nnf_report` never named (no entry in `forms`) renders as
/// an empty `NecessaryNormalForm` — see `lookup`'s own doc comment for
/// why that's not an error.
fn normal_form_property(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    code: SctId,
    nnf_report: Option<&NecessaryNormalFormReport>,
    property_name: &str,
    terse: bool,
) -> Result<String, FhirError> {
    let report =
        nnf_report.ok_or_else(|| FhirError::MissingClassification(property_name.to_string()))?;
    let empty = NecessaryNormalForm::default();
    let nnf = report.forms.get(&code).unwrap_or(&empty);
    Ok(normal_form::render(store, language_refset, nnf, terse))
}

/// The preferred term in `language_refset` if one was given and found,
/// else the active FSN, else `None`. Shared by `lookup`'s `display` and
/// `expand`'s per-`contains`-entry display (spec/11).
pub(crate) fn display_for(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    concept_id: SctId,
) -> Option<String> {
    language_refset
        .and_then(|refset| store.preferred_term(concept_id, refset))
        .or_else(|| store.fsn(concept_id))
        .map(|d| d.term.clone())
}

/// Every active FSN/synonym description of `concept_id` (`TextDefinition`
/// rows excluded — `lookup`'s `definition` field covers those), each with
/// its use in `language_refset`. Shared by `lookup` and `expand`'s
/// `includeDesignations` (spec/11).
pub(crate) fn designations_for(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    concept_id: SctId,
) -> Vec<Designation> {
    store
        .descriptions_of(concept_id)
        .filter(|d| d.active && d.type_id != constants::TEXT_DEFINITION)
        .map(|d| Designation {
            language_code: d.language_code.clone(),
            value: d.term.clone(),
            use_: designation_use(store, language_refset, d.id),
        })
        .collect()
}

fn designation_use(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    description_id: SctId,
) -> DesignationUse {
    let Some(refset) = language_refset else {
        return DesignationUse::Unspecified;
    };
    match store.acceptability(refset, description_id) {
        Some(id) if id == constants::PREFERRED => DesignationUse::Preferred,
        Some(id) if id == constants::ACCEPTABLE => DesignationUse::Acceptable,
        _ => DesignationUse::Unspecified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::{Concept, Description};
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;
    use snomed_rf2::refset::{LanguageRefsetMember, RefsetMemberCore};

    const FINDING: SctId = constants::ROOT_CONCEPT;

    fn concept(id: SctId, active: bool) -> Concept {
        Concept {
            id,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    fn description(id: u64, concept_id: SctId, type_id: SctId, term: &str) -> Description {
        Description {
            id: SctId::compose(id, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: "en".to_string(),
            type_id,
            term: term.to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        }
    }

    fn language_member(
        uuid: &str,
        description_id: SctId,
        acceptability_id: SctId,
    ) -> LanguageRefsetMember {
        LanguageRefsetMember {
            core: RefsetMemberCore {
                id: uuid.to_string(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: description_id,
            },
            acceptability_id,
        }
    }

    fn fixture() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING, true));
        let fsn = description(
            2001,
            FINDING,
            constants::FULLY_SPECIFIED_NAME,
            "SNOMED CT Concept (SNOMED RT+CTV3)",
        );
        let syn = description(2002, FINDING, constants::SYNONYM, "SNOMED CT Concept");
        let def = description(
            2003,
            FINDING,
            constants::TEXT_DEFINITION,
            "The root concept of SNOMED CT.",
        );
        b.add_language_member(language_member(
            "80000000-0000-4000-8000-000000000021",
            syn.id,
            constants::PREFERRED,
        ));
        b.add_description(fsn);
        b.add_description(syn);
        b.add_description(def);
        b.build()
    }

    #[test]
    fn rejects_a_non_snomed_system() {
        let store = fixture();
        let err = lookup(&store, "http://loinc.org", FINDING, None, None, &[], None).unwrap_err();
        assert!(matches!(err, FhirError::UnsupportedSystem(s) if s == "http://loinc.org"));
    }

    #[test]
    fn rejects_an_unknown_code() {
        let store = fixture();
        let unknown = SctId::compose(9999, ComponentType::Concept, None).unwrap();
        let err = lookup(&store, SNOMED_CT_SYSTEM, unknown, None, None, &[], None).unwrap_err();
        assert_eq!(err, FhirError::UnknownCode(unknown));
    }

    #[test]
    fn display_falls_back_to_fsn_without_a_language_refset() {
        let store = fixture();
        let result = lookup(&store, SNOMED_CT_SYSTEM, FINDING, None, None, &[], None).unwrap();
        assert_eq!(
            result.display.as_deref(),
            Some("SNOMED CT Concept (SNOMED RT+CTV3)")
        );
    }

    #[test]
    fn display_prefers_the_preferred_term_in_the_given_refset() {
        let store = fixture();
        let result = lookup(
            &store,
            SNOMED_CT_SYSTEM,
            FINDING,
            None,
            Some(constants::US_ENGLISH_LANGUAGE_REFSET),
            &[],
            None,
        )
        .unwrap();
        assert_eq!(result.display.as_deref(), Some("SNOMED CT Concept"));
    }

    #[test]
    fn definition_reads_the_text_definition_row() {
        let store = fixture();
        let result = lookup(&store, SNOMED_CT_SYSTEM, FINDING, None, None, &[], None).unwrap();
        assert_eq!(
            result.definition.as_deref(),
            Some("The root concept of SNOMED CT.")
        );
    }

    #[test]
    fn designation_excludes_text_definition_and_marks_acceptability() {
        let store = fixture();
        let result = lookup(
            &store,
            SNOMED_CT_SYSTEM,
            FINDING,
            None,
            Some(constants::US_ENGLISH_LANGUAGE_REFSET),
            &[],
            None,
        )
        .unwrap();
        assert_eq!(result.designation.len(), 2, "{:?}", result.designation);
        let syn = result
            .designation
            .iter()
            .find(|d| d.value == "SNOMED CT Concept")
            .unwrap();
        assert_eq!(syn.use_, DesignationUse::Preferred);
        let fsn = result
            .designation
            .iter()
            .find(|d| d.value.starts_with("SNOMED CT Concept ("))
            .unwrap();
        assert_eq!(fsn.use_, DesignationUse::Unspecified);
    }

    #[test]
    fn default_properties_are_returned_when_none_requested() {
        let store = fixture();
        let result = lookup(&store, SNOMED_CT_SYSTEM, FINDING, None, None, &[], None).unwrap();
        let codes: Vec<&str> = result.property.iter().map(|p| p.code()).collect();
        assert_eq!(codes, vec!["inactive", "moduleId", "sufficientlyDefined"]);
        assert!(result.property.contains(&LookupProperty::Inactive(false)));
    }

    #[test]
    fn a_specific_property_can_be_requested() {
        let store = fixture();
        let result = lookup(
            &store,
            SNOMED_CT_SYSTEM,
            FINDING,
            None,
            None,
            &["moduleId"],
            None,
        )
        .unwrap();
        assert_eq!(
            result.property,
            vec![LookupProperty::ModuleId(constants::CORE_MODULE)]
        );
    }

    #[test]
    fn rejects_an_unsupported_property() {
        let store = fixture();
        let err = lookup(
            &store,
            SNOMED_CT_SYSTEM,
            FINDING,
            None,
            None,
            &["notARealProperty"],
            None,
        )
        .unwrap_err();
        assert_eq!(
            err,
            FhirError::UnsupportedProperty("notARealProperty".to_string())
        );
    }

    #[test]
    fn rejects_normal_form_without_a_report() {
        let store = fixture();
        for name in ["normalForm", "normalFormTerse"] {
            let err =
                lookup(&store, SNOMED_CT_SYSTEM, FINDING, None, None, &[name], None).unwrap_err();
            assert_eq!(err, FhirError::MissingClassification(name.to_string()));
        }
    }

    #[test]
    fn normal_form_properties_render_when_a_report_is_supplied() {
        use std::collections::HashMap;

        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING, true));
        let parent = SctId::compose(9101, ComponentType::Concept, None).unwrap();
        b.add_concept(concept(parent, true));
        b.add_description(description(
            2010,
            parent,
            constants::FULLY_SPECIFIED_NAME,
            "Parent concept (finding)",
        ));
        let store = b.build();

        let mut forms = HashMap::new();
        forms.insert(
            FINDING,
            snomed_classify::NecessaryNormalForm {
                is_a: vec![parent],
                attributes: vec![],
            },
        );
        let report = NecessaryNormalFormReport {
            forms,
            skipped: Vec::new(),
        };

        let result = lookup(
            &store,
            SNOMED_CT_SYSTEM,
            FINDING,
            None,
            None,
            &["normalForm", "normalFormTerse"],
            Some(&report),
        )
        .unwrap();
        assert_eq!(
            result.property,
            vec![
                LookupProperty::NormalForm(format!("{parent} |Parent concept (finding)|")),
                LookupProperty::NormalFormTerse(parent.to_string()),
            ]
        );

        // A concept the report never named renders as an empty
        // expression, not an error.
        let unnamed = SctId::compose(9102, ComponentType::Concept, None).unwrap();
        let mut store2 = SnapshotStore::builder();
        store2.add_concept(concept(unnamed, true));
        let store2 = store2.build();
        let result = lookup(
            &store2,
            SNOMED_CT_SYSTEM,
            unnamed,
            None,
            None,
            &["normalForm"],
            Some(&report),
        )
        .unwrap();
        assert_eq!(
            result.property,
            vec![LookupProperty::NormalForm(String::new())]
        );
    }

    #[test]
    fn version_is_passed_through_verbatim() {
        let store = fixture();
        let result = lookup(
            &store,
            SNOMED_CT_SYSTEM,
            FINDING,
            Some("http://snomed.info/sct/900000000000207008/version/20250801"),
            None,
            &[],
            None,
        )
        .unwrap();
        assert_eq!(
            result.version.as_deref(),
            Some("http://snomed.info/sct/900000000000207008/version/20250801")
        );
        assert_eq!(result.name, "SNOMED CT");
    }
}
