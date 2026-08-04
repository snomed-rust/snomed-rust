//! Necessary normal form generation, per `spec/14-necessary-normal-form.md`:
//! reduces a classified ontology's entailed subsumption plus each
//! concept's stated attributes down to the minimal set that, together
//! with subsumption reasoning, implies everything else — the shape real
//! RF2 `Relationship` rows take.

use std::collections::{HashMap, HashSet};

use snomed_core::sctid::SctId;
use snomed_owl::Axiom;

use crate::skipped::SkippedConstruct;
use crate::stated_profile::{self, Attribute as RawAttribute, StatedProfile};
use crate::{classify, normalize, Classification};

/// One necessary-normal-form relationship: `group` is `0` for ungrouped
/// (spec/07's `relationshipGroup` convention), otherwise a group number
/// assigned `1..N` in a stable (sorted) order — this crate generates from
/// a single axiom set with no prior release to number against, so there's
/// no attempt to preserve any particular numbering scheme beyond that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attribute {
    pub group: u32,
    pub type_id: SctId,
    pub destination_id: SctId,
}

/// A concept's necessary normal form: its proximal (most specific
/// entailed, non-redundant) parents, and its redundancy-reduced
/// attributes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NecessaryNormalForm {
    pub is_a: Vec<SctId>,
    pub attributes: Vec<Attribute>,
}

/// [`necessary_normal_form`]'s result: one [`NecessaryNormalForm`] per
/// named concept the input axioms said anything about, plus every
/// construct recognized but not modeled (spec/14's scope), reported never
/// silently dropped.
#[derive(Debug, Clone)]
pub struct NecessaryNormalFormReport {
    pub forms: HashMap<SctId, NecessaryNormalForm>,
    pub skipped: Vec<SkippedConstruct>,
}

/// Computes the necessary normal form of every concept `axioms` mention,
/// via classification (spec/13) plus stated-profile redundancy
/// elimination (spec/14). `axioms` is read multiple times (classification,
/// stated-profile extraction, role hierarchy) — a slice, not a
/// single-pass iterator, unlike [`classify`].
pub fn necessary_normal_form(axioms: &[Axiom]) -> NecessaryNormalFormReport {
    let classification_report = classify(axioms);
    let classification = classification_report.classification;

    let (profiles, mut skipped) = stated_profile::extract_stated_profiles(axioms);
    skipped.extend(classification_report.skipped.iter().copied());
    dedup_unordered(&mut skipped);

    let role_ancestors = role_ancestor_closure(axioms);

    let mut concepts: HashSet<SctId> = classification.concepts().collect();
    concepts.extend(profiles.keys().copied());

    let proximal: HashMap<SctId, Vec<SctId>> = concepts
        .iter()
        .map(|&c| (c, proximal_parents(c, &classification)))
        .collect();

    let mut ctx = Context {
        profiles: &profiles,
        classification: &classification,
        role_ancestors: &role_ancestors,
        proximal: &proximal,
        cache: HashMap::new(),
        in_progress: HashSet::new(),
    };

    let mut forms = HashMap::new();
    for &c in &concepts {
        let candidates = groups_for(c, &mut ctx);
        let is_a = proximal.get(&c).cloned().unwrap_or_default();
        forms.insert(c, finalize(is_a, candidates));
    }

    NecessaryNormalFormReport { forms, skipped }
}

fn dedup_unordered<T: PartialEq + Copy>(items: &mut Vec<T>) {
    let mut unique = Vec::with_capacity(items.len());
    for item in items.drain(..) {
        if !unique.contains(&item) {
            unique.push(item);
        }
    }
    *items = unique;
}

/// `p` survives as a proximal parent of `c` unless some other entailed
/// supertype `q` of `c` already implies `p` (spec/14 rule 1).
fn proximal_parents(c: SctId, classification: &Classification) -> Vec<SctId> {
    let all: Vec<SctId> = classification.subsumers(c).collect();
    all.iter()
        .filter(|&&p| {
            !all.iter()
                .any(|&q| q != p && classification.is_subsumed_by(q, p))
        })
        .copied()
        .collect()
}

/// Transitive closure of `SubObjectPropertyOf`'s plain (non-chain) role
/// hierarchy edges, reflexive (`r` is its own ancestor) so callers never
/// need a separate `==` check alongside a closure lookup.
fn role_ancestor_closure(axioms: &[Axiom]) -> HashMap<SctId, HashSet<SctId>> {
    let tbox = normalize::normalize(axioms);
    let mut direct: HashMap<SctId, Vec<SctId>> = HashMap::new();
    for (sub, sup) in &tbox.role_hierarchy {
        if let (crate::types::RoleId::Named(s), crate::types::RoleId::Named(t)) = (sub, sup) {
            direct.entry(*s).or_default().push(*t);
        }
    }

    let mut closure: HashMap<SctId, HashSet<SctId>> = HashMap::new();
    for &role in direct.keys() {
        let mut seen = HashSet::from([role]);
        let mut queue = vec![role];
        while let Some(r) = queue.pop() {
            for &next in direct.get(&r).map(Vec::as_slice).unwrap_or(&[]) {
                if seen.insert(next) {
                    queue.push(next);
                }
            }
        }
        closure.insert(role, seen);
    }
    closure
}

struct Context<'a> {
    profiles: &'a HashMap<SctId, StatedProfile>,
    classification: &'a Classification,
    role_ancestors: &'a HashMap<SctId, HashSet<SctId>>,
    proximal: &'a HashMap<SctId, Vec<SctId>>,
    cache: HashMap<SctId, Vec<GroupCandidate>>,
    in_progress: HashSet<SctId>,
}

#[derive(Debug, Clone)]
struct GroupCandidate {
    /// `true` if this candidate originated as (or was inherited from) an
    /// ungrouped (`relationshipGroup 0`) stated attribute.
    group0: bool,
    fragments: Vec<RawAttribute>,
}

/// Computes (and caches) `c`'s redundancy-reduced attribute groups:
/// its own stated groups plus each proximal parent's already-reduced
/// groups, combined via [`insert_candidate`]. Cycle-safe: a concept
/// reachable from itself via proximal-parent edges (only possible via a
/// degenerate `EquivalentClasses` cycle — spec/14 rule 3) contributes
/// nothing rather than recursing forever.
fn groups_for(c: SctId, ctx: &mut Context<'_>) -> Vec<GroupCandidate> {
    if let Some(cached) = ctx.cache.get(&c) {
        return cached.clone();
    }
    if !ctx.in_progress.insert(c) {
        return Vec::new();
    }

    let mut candidates: Vec<GroupCandidate> = Vec::new();

    if let Some(profile) = ctx.profiles.get(&c) {
        for &attr in &profile.ungrouped {
            insert_candidate(
                &mut candidates,
                GroupCandidate {
                    group0: true,
                    fragments: vec![attr],
                },
                ctx.role_ancestors,
                ctx.classification,
            );
        }
        for group in profile.groups.clone() {
            insert_candidate(
                &mut candidates,
                GroupCandidate {
                    group0: false,
                    fragments: group,
                },
                ctx.role_ancestors,
                ctx.classification,
            );
        }
    }

    let parents = ctx.proximal.get(&c).cloned().unwrap_or_default();
    for parent in parents {
        for inherited in groups_for(parent, ctx) {
            insert_candidate(
                &mut candidates,
                inherited,
                ctx.role_ancestors,
                ctx.classification,
            );
        }
    }

    ctx.in_progress.remove(&c);
    ctx.cache.insert(c, candidates.clone());
    candidates
}

/// `GroupSet.add`, ported: `new` is dropped if some existing candidate is
/// already same-or-stronger; otherwise `new` replaces every existing
/// candidate `new` makes redundant.
fn insert_candidate(
    candidates: &mut Vec<GroupCandidate>,
    new: GroupCandidate,
    role_ancestors: &HashMap<SctId, HashSet<SctId>>,
    classification: &Classification,
) {
    if candidates
        .iter()
        .any(|g| group_is_same_or_stronger(g, &new, role_ancestors, classification))
    {
        return;
    }
    candidates.retain(|g| !group_is_same_or_stronger(&new, g, role_ancestors, classification));
    candidates.push(new);
}

/// `candidate` is same-or-stronger than `other` when every fragment in
/// `other` is made redundant by some fragment in `candidate` — `candidate`
/// may have extra fragments `other` doesn't need to cover.
fn group_is_same_or_stronger(
    candidate: &GroupCandidate,
    other: &GroupCandidate,
    role_ancestors: &HashMap<SctId, HashSet<SctId>>,
    classification: &Classification,
) -> bool {
    other.fragments.iter().all(|&weaker| {
        candidate.fragments.iter().any(|&stronger| {
            fragment_is_same_or_stronger(stronger, weaker, role_ancestors, classification)
        })
    })
}

/// `(s, D)` makes `(r, C)` redundant when `r` is `s` or an ancestor of `s`
/// in the role hierarchy, and `C` is `D` or an ancestor of `D` in concept
/// subsumption (spec/14).
fn fragment_is_same_or_stronger(
    stronger: RawAttribute,
    weaker: RawAttribute,
    role_ancestors: &HashMap<SctId, HashSet<SctId>>,
    classification: &Classification,
) -> bool {
    let (s, d) = stronger;
    let (r, c) = weaker;

    let type_ok = s == r
        || role_ancestors
            .get(&s)
            .is_some_and(|ancestors| ancestors.contains(&r));
    if !type_ok {
        return false;
    }

    d == c || classification.is_subsumed_by(d, c)
}

fn finalize(mut is_a: Vec<SctId>, candidates: Vec<GroupCandidate>) -> NecessaryNormalForm {
    is_a.sort();
    is_a.dedup();

    let mut ungrouped: Vec<RawAttribute> = Vec::new();
    let mut numbered_groups: Vec<Vec<RawAttribute>> = Vec::new();
    for candidate in candidates {
        if candidate.group0 {
            ungrouped.extend(candidate.fragments);
        } else {
            numbered_groups.push(candidate.fragments);
        }
    }
    ungrouped.sort();

    // Stable, deterministic numbering: sort groups by their fragments.
    for group in &mut numbered_groups {
        group.sort();
    }
    numbered_groups.sort();

    let mut attributes: Vec<Attribute> = ungrouped
        .into_iter()
        .map(|(type_id, destination_id)| Attribute {
            group: 0,
            type_id,
            destination_id,
        })
        .collect();
    for (index, group) in numbered_groups.into_iter().enumerate() {
        let group_number = (index + 1) as u32;
        attributes.extend(
            group
                .into_iter()
                .map(|(type_id, destination_id)| Attribute {
                    group: group_number,
                    type_id,
                    destination_id,
                }),
        );
    }

    NecessaryNormalForm { is_a, attributes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::sctid::ComponentType;

    /// A synthetic, check-digit-valid SCTID for test fixture concepts
    /// that aren't genuine SNOMED CT concepts (root `CLAUDE.md` convention).
    fn id(item: u64) -> SctId {
        SctId::compose(item, ComponentType::Concept, None).unwrap()
    }

    fn ax(s: &str) -> Axiom {
        snomed_owl::parse(s).unwrap_or_else(|e| panic!("failed to parse {s:?}: {e}"))
    }

    fn axioms(strs: &[String]) -> Vec<Axiom> {
        strs.iter().map(|s| ax(s)).collect()
    }

    #[test]
    fn proximal_parents_exclude_transitively_redundant_ancestors() {
        let a = id(2001);
        let b = id(2002);
        let c = id(2003);
        let input = axioms(&[
            format!("SubClassOf(:{a} :{b})"),
            format!("SubClassOf(:{b} :{c})"),
        ]);
        let report = necessary_normal_form(&input);

        // A is entailed to be a subtype of both B and C, but C is
        // redundant (implied transitively via B) — only B survives.
        let nnf_a = &report.forms[&a];
        assert_eq!(nnf_a.is_a, vec![b]);
        let nnf_b = &report.forms[&b];
        assert_eq!(nnf_b.is_a, vec![c]);
    }

    #[test]
    fn own_specific_attribute_makes_inherited_general_one_redundant() {
        // Parent has `site = General`; Child (a subtype of Parent) states
        // its own `site = Specific`, where Specific is a subtype of
        // General. Child's NNF must keep only the specific attribute —
        // the inherited general one is implied by it.
        let parent = id(2010);
        let child = id(2011);
        let site = id(2012);
        let general = id(2013);
        let specific = id(2014);
        let input = axioms(&[
            format!("SubClassOf(:{specific} :{general})"),
            format!("SubClassOf(:{parent} ObjectSomeValuesFrom(:{site} :{general}))"),
            format!("SubClassOf(:{child} :{parent})"),
            format!("SubClassOf(:{child} ObjectSomeValuesFrom(:{site} :{specific}))"),
        ]);
        let report = necessary_normal_form(&input);

        let nnf_child = &report.forms[&child];
        assert_eq!(
            nnf_child.attributes,
            vec![Attribute {
                group: 0,
                type_id: site,
                destination_id: specific
            }]
        );
    }

    #[test]
    fn role_hierarchy_makes_general_attribute_type_redundant() {
        // Parent has `genericAttr = V`; Child states its own
        // `specificAttr = V`, where specificAttr is a subtype (in the
        // role hierarchy) of genericAttr. Child's NNF keeps only the
        // specific-typed attribute.
        let parent = id(2020);
        let child = id(2021);
        let generic_attr = id(2022);
        let specific_attr = id(2023);
        let value = id(2024);
        let input = axioms(&[
            format!("SubObjectPropertyOf(:{specific_attr} :{generic_attr})"),
            format!("SubClassOf(:{parent} ObjectSomeValuesFrom(:{generic_attr} :{value}))"),
            format!("SubClassOf(:{child} :{parent})"),
            format!("SubClassOf(:{child} ObjectSomeValuesFrom(:{specific_attr} :{value}))"),
        ]);
        let report = necessary_normal_form(&input);

        let nnf_child = &report.forms[&child];
        assert_eq!(
            nnf_child.attributes,
            vec![Attribute {
                group: 0,
                type_id: specific_attr,
                destination_id: value
            }]
        );
    }

    #[test]
    fn role_group_is_reconstructed_from_the_owl_encoding() {
        // The real SNOMED OWL pattern (spec/12, spec/14): a role group is
        // ObjectSomeValuesFrom(RoleGroup, ObjectIntersectionOf(attrs...)).
        let concept = id(2030);
        let parent = id(2031);
        let finding_site = id(2032);
        let site_value = id(2033);
        let morphology = id(2034);
        let morphology_value = id(2035);
        let input = axioms(&[format!(
            "EquivalentClasses(:{concept} ObjectIntersectionOf(:{parent} \
             ObjectSomeValuesFrom(:609096000 ObjectIntersectionOf(\
             ObjectSomeValuesFrom(:{finding_site} :{site_value}) \
             ObjectSomeValuesFrom(:{morphology} :{morphology_value})))))"
        )]);
        let report = necessary_normal_form(&input);

        let nnf = &report.forms[&concept];
        assert_eq!(nnf.is_a, vec![parent]);
        let mut attrs = nnf.attributes.clone();
        attrs.sort_by_key(|a| a.type_id);
        assert_eq!(
            attrs,
            vec![
                Attribute {
                    group: 1,
                    type_id: finding_site,
                    destination_id: site_value
                },
                Attribute {
                    group: 1,
                    type_id: morphology,
                    destination_id: morphology_value
                },
            ]
        );
    }

    #[test]
    fn ungrouped_attribute_stays_group_zero() {
        // A top-level ObjectSomeValuesFrom, not wrapped in RoleGroup, is
        // an MRCM-never-grouped-style attribute — relationshipGroup 0.
        let concept = id(2040);
        let attr = id(2041);
        let value = id(2042);
        let input = axioms(&[format!(
            "SubClassOf(:{concept} ObjectSomeValuesFrom(:{attr} :{value}))"
        )]);
        let report = necessary_normal_form(&input);

        let nnf = &report.forms[&concept];
        assert_eq!(
            nnf.attributes,
            vec![Attribute {
                group: 0,
                type_id: attr,
                destination_id: value
            }]
        );
    }

    #[test]
    fn group_redundant_across_two_whole_groups_is_eliminated() {
        // Group 1 (inherited): { siteAttr = GeneralSite }.
        // Group 2 (own, more specific): { siteAttr = SpecificSite, extra = X }.
        // Since SpecificSite ⊑ GeneralSite, group 2 alone entails
        // everything group 1 requires — group 1 is fully redundant.
        let parent = id(2050);
        let child = id(2051);
        let site_attr = id(2052);
        let general_site = id(2053);
        let specific_site = id(2054);
        let extra_attr = id(2055);
        let extra_value = id(2056);
        let input = axioms(&[
            format!("SubClassOf(:{specific_site} :{general_site})"),
            format!(
                "SubClassOf(:{parent} ObjectSomeValuesFrom(:609096000 \
                 ObjectSomeValuesFrom(:{site_attr} :{general_site})))"
            ),
            format!("SubClassOf(:{child} :{parent})"),
            format!(
                "SubClassOf(:{child} ObjectSomeValuesFrom(:609096000 ObjectIntersectionOf(\
                 ObjectSomeValuesFrom(:{site_attr} :{specific_site}) \
                 ObjectSomeValuesFrom(:{extra_attr} :{extra_value}))))"
            ),
        ]);
        let report = necessary_normal_form(&input);

        let nnf_child = &report.forms[&child];
        let mut attrs = nnf_child.attributes.clone();
        attrs.sort_by_key(|a| a.type_id);
        assert_eq!(
            attrs,
            vec![
                Attribute {
                    group: 1,
                    type_id: site_attr,
                    destination_id: specific_site
                },
                Attribute {
                    group: 1,
                    type_id: extra_attr,
                    destination_id: extra_value
                },
            ]
        );
    }

    #[test]
    fn unmodeled_attribute_shape_is_reported_not_silently_dropped() {
        // A DataHasValue filler inside a role group isn't modeled.
        let concept = id(2060);
        let attr = id(2061);
        let value_attr = id(2062);
        let input = axioms(&[format!(
            "SubClassOf(:{concept} ObjectSomeValuesFrom(:609096000 \
             DataHasValue(:{value_attr} \"1\"^^xsd:integer)))"
        )]);
        let report = necessary_normal_form(&input);

        let nnf = &report.forms[&concept];
        assert!(nnf.attributes.is_empty());
        assert!(report
            .skipped
            .contains(&SkippedConstruct::UnmodeledAttributeShape { concept }));
        let _ = attr; // referenced for readability of the fixture only
    }

    #[test]
    fn gci_contributes_only_via_subsumption_never_a_direct_profile() {
        // SubClassOf(ObjectIntersectionOf(A, B), C) — a GCI. X, defined as
        // A AND B, is thereby entailed a subtype of C and inherits C's
        // stated attribute, purely through classification.
        let a = id(2070);
        let b = id(2071);
        let c = id(2072);
        let x = id(2073);
        let attr = id(2074);
        let value = id(2075);
        let input = axioms(&[
            format!("SubClassOf(ObjectIntersectionOf(:{a} :{b}) :{c})"),
            format!("SubClassOf(:{c} ObjectSomeValuesFrom(:{attr} :{value}))"),
            format!("EquivalentClasses(:{x} ObjectIntersectionOf(:{a} :{b}))"),
        ]);
        let report = necessary_normal_form(&input);

        let nnf_x = &report.forms[&x];
        assert!(nnf_x.is_a.contains(&c));
        assert_eq!(
            nnf_x.attributes,
            vec![Attribute {
                group: 0,
                type_id: attr,
                destination_id: value
            }]
        );
    }
}
