//! Necessary normal form generation, per `spec/14-necessary-normal-form.md`:
//! reduces a classified ontology's entailed subsumption plus each
//! concept's stated attributes down to the minimal set that, together
//! with subsumption reasoning, implies everything else — the shape real
//! RF2 `Relationship` rows take.

use std::collections::{HashMap, HashSet};

use snomed_core::sctid::SctId;
use snomed_owl::{Axiom, ObjectPropertyExpression};

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
        chains: None,
    };

    // First pass: Rule 1 only (class and role inclusions). Rule 2 can't run
    // yet — it asks whether one attribute's filler reaches another's by
    // following a property, and that graph is made of the very forms this
    // pass produces (spec/14).
    let mut forms = HashMap::new();
    for &c in &concepts {
        let candidates = groups_for(c, &mut ctx);
        let is_a = proximal.get(&c).cloned().unwrap_or_default();
        forms.insert(c, finalize(is_a, candidates));
    }

    // Second pass: with the node graphs built, re-normalize the concepts
    // Rule 2 could possibly affect — those holding an attribute whose type
    // is (or is a subtype of) some chain's source type. Everything else
    // would recompute to the identical answer, so the reference
    // implementation skips it too.
    let chains = property_chains(axioms);
    if !chains.is_empty() {
        let graphs = NodeGraphs::build(&chains, &forms);
        let affected: Vec<SctId> = forms
            .iter()
            .filter(|(_, form)| {
                form.attributes.iter().any(|attribute| {
                    chains.iter().any(|chain| {
                        chain.source == attribute.type_id
                            || role_ancestors
                                .get(&attribute.type_id)
                                .is_some_and(|a| a.contains(&chain.source))
                    })
                })
            })
            .map(|(&c, _)| c)
            .collect();

        ctx.chains = Some((&chains, &graphs));
        for c in affected {
            // Drop only this concept's cached first-pass groups: its
            // ancestors' entries stay, so inherited fragments arrive
            // already reduced, exactly as in the reference.
            ctx.cache.remove(&c);
            let candidates = groups_for(c, &mut ctx);
            let is_a = proximal.get(&c).cloned().unwrap_or_default();
            forms.insert(c, finalize(is_a, candidates));
        }
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
///
/// Two *equivalent* supertypes imply each other, so a naive "drop `p` when
/// some `q` implies it" eliminates both and leaves `c` with no parent at
/// all (spec/14 rule 5). Equivalent supertypes are therefore an
/// equivalence class from which exactly one representative survives — the
/// lowest SCTID, so the choice is deterministic rather than dependent on
/// iteration order.
fn proximal_parents(c: SctId, classification: &Classification) -> Vec<SctId> {
    let all: Vec<SctId> = classification.subsumers(c).collect();
    all.iter()
        .filter(|&&p| {
            !all.iter().any(|&q| {
                if q == p || !classification.is_subsumed_by(q, p) {
                    return false;
                }
                // `q` implies `p`. If `p` implies `q` back they are
                // equivalent, and only the lower id survives; otherwise
                // `q` is strictly more specific and `p` is redundant.
                !classification.is_subsumed_by(p, q) || q < p
            })
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

/// One `SubObjectPropertyOf(ObjectPropertyChain(t s) r)` axiom, in the
/// reference implementation's names: `t ∘ s ⊑ r` (spec/14 Rule 2).
///
/// A `TransitiveObjectProperty(r)` axiom is the chain `r ∘ r ⊑ r`, and is
/// collected here as one — which is why transitive-property redundancy
/// needs no separate rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PropertyChain {
    source: SctId,
    destination: SctId,
    inferred: SctId,
}

/// Every property chain in `axioms`. Chains longer than two operands are
/// skipped: `snomed-classify` normalizes them for *classification* with
/// fresh intermediate roles (spec/13), but those fresh roles name nothing
/// a relationship can refer to, so they can't participate in the
/// relationship-level redundancy Rule 2 describes. SNOMED CT uses only
/// two-operand chains (spec/12).
fn property_chains(axioms: &[Axiom]) -> Vec<PropertyChain> {
    let mut chains = Vec::new();
    for axiom in axioms {
        match axiom {
            Axiom::SubObjectPropertyOf {
                sub: ObjectPropertyExpression::Chain(ids),
                sup,
            } if ids.len() == 2 => chains.push(PropertyChain {
                source: ids[0],
                destination: ids[1],
                inferred: *sup,
            }),
            Axiom::TransitiveObjectProperty(r) => chains.push(PropertyChain {
                source: *r,
                destination: *r,
                inferred: *r,
            }),
            _ => {}
        }
    }
    chains.sort_by_key(|c| (c.source, c.destination, c.inferred));
    chains.dedup();
    chains
}

/// Direct `concept --property--> filler` edges for every property that
/// appears as a chain's *destination*, taken from the first pass's normal
/// forms — the reference implementation's `NodeGraph` per traversable
/// property.
///
/// Only chain destinations need a graph: Rule 2 asks "does `D` reach `C`
/// via `s`", and `s` is always a chain's destination type.
#[derive(Debug, Default)]
struct NodeGraphs {
    edges: HashMap<(SctId, SctId), Vec<SctId>>,
}

impl NodeGraphs {
    fn build(chains: &[PropertyChain], forms: &HashMap<SctId, NecessaryNormalForm>) -> Self {
        let traversable: HashSet<SctId> = chains.iter().map(|c| c.destination).collect();
        let mut edges: HashMap<(SctId, SctId), Vec<SctId>> = HashMap::new();
        for (&concept, form) in forms {
            for attribute in &form.attributes {
                if traversable.contains(&attribute.type_id) {
                    edges
                        .entry((concept, attribute.type_id))
                        .or_default()
                        .push(attribute.destination_id);
                }
            }
        }
        for targets in edges.values_mut() {
            targets.sort_unstable();
            targets.dedup();
        }
        NodeGraphs { edges }
    }

    /// The reference's `getPropertyChainTransitiveClosure`: everything
    /// reachable from `from` by following `property` any number of times,
    /// including `from` itself, plus the concept-subsumption ancestors of
    /// every concept so reached.
    fn chain_closure(
        &self,
        from: SctId,
        property: SctId,
        classification: &Classification,
    ) -> HashSet<SctId> {
        let mut reached = HashSet::from([from]);
        let mut queue = vec![from];
        while let Some(node) = queue.pop() {
            for &next in self
                .edges
                .get(&(node, property))
                .map(Vec::as_slice)
                .unwrap_or(&[])
            {
                if reached.insert(next) {
                    queue.push(next);
                }
            }
        }
        let ancestors: Vec<SctId> = reached
            .iter()
            .flat_map(|&node| classification.subsumers(node))
            .collect();
        reached.extend(ancestors);
        reached
    }
}

struct Context<'a> {
    profiles: &'a HashMap<SctId, StatedProfile>,
    classification: &'a Classification,
    role_ancestors: &'a HashMap<SctId, HashSet<SctId>>,
    proximal: &'a HashMap<SctId, Vec<SctId>>,
    cache: HashMap<SctId, Vec<GroupCandidate>>,
    in_progress: HashSet<SctId>,
    /// `None` during the first pass, `Some` during the second: Rule 2
    /// needs the node graphs, and those can only be built once every
    /// concept's first-pass form is known (spec/14).
    chains: Option<(&'a [PropertyChain], &'a NodeGraphs)>,
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
                ctx.chains,
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
                ctx.chains,
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
                ctx.chains,
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
    chains: Option<(&[PropertyChain], &NodeGraphs)>,
) {
    if candidates
        .iter()
        .any(|g| group_is_same_or_stronger(g, &new, role_ancestors, classification, chains))
    {
        return;
    }
    candidates
        .retain(|g| !group_is_same_or_stronger(&new, g, role_ancestors, classification, chains));
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
    chains: Option<(&[PropertyChain], &NodeGraphs)>,
) -> bool {
    other.fragments.iter().all(|&weaker| {
        candidate.fragments.iter().any(|&stronger| {
            fragment_is_same_or_stronger(stronger, weaker, role_ancestors, classification, chains)
        })
    })
}

/// Whether `stronger` makes `weaker` redundant, by either of spec/14's two
/// rules. Named after the reference implementation's
/// `RelationshipFragment.isSameOrStrongerThan`, and following its
/// vocabulary: `stronger` is `B = (u, D)`, `weaker` is `A = (r, C)`.
///
/// **Rule 1 — class and role inclusions.** `A` is redundant when `u` is
/// `r` or a subtype of it, and `D` is `C` or a subtype of it.
///
/// **Rule 2 — property chains.** Given a chain `t ∘ s ⊑ r`, `A` is
/// redundant when `u` is `t` or a subtype of it, and `D` reaches `C` via
/// `s`. Concretely: if a concept has `findingSite = Hand` and
/// `findingSite = UpperLimb`, `Hand partOf UpperLimb` holds, and
/// `findingSite ∘ partOf ⊑ findingSite`, then the second attribute adds
/// nothing the first doesn't already imply. Only available on the second
/// pass, when `chains` is `Some` — the node graphs it needs are built from
/// the first pass's forms.
fn fragment_is_same_or_stronger(
    stronger: RawAttribute,
    weaker: RawAttribute,
    role_ancestors: &HashMap<SctId, HashSet<SctId>>,
    classification: &Classification,
    chains: Option<(&[PropertyChain], &NodeGraphs)>,
) -> bool {
    let (u, d) = stronger;
    let (r, c) = weaker;
    let role_closure = |role: SctId, ancestor: SctId| {
        role == ancestor
            || role_ancestors
                .get(&role)
                .is_some_and(|ancestors| ancestors.contains(&ancestor))
    };

    // Rule 1.
    if role_closure(u, r) && (d == c || classification.is_subsumed_by(d, c)) {
        return true;
    }

    // Rule 2.
    let Some((chains, graphs)) = chains else {
        return false;
    };
    chains
        .iter()
        .filter(|chain| chain.inferred == r && role_closure(u, chain.source))
        .any(|chain| {
            graphs
                .chain_closure(d, chain.destination, classification)
                .contains(&c)
        })
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

    #[test]
    fn equivalent_parents_do_not_eliminate_each_other() {
        // spec/14 rule 5: `B` and `C` are equivalent, so each implies the
        // other. Dropping every implied parent would leave `A` with no
        // parent at all; exactly one representative (the lower SCTID)
        // must survive.
        let a = id(1090);
        let b = id(1091);
        let c = id(1092);
        assert!(b < c, "the test relies on b sorting before c");
        let report = necessary_normal_form(&[
            ax(&format!("EquivalentClasses(:{b} :{c})")),
            ax(&format!("SubClassOf(:{a} :{b})")),
        ]);
        assert_eq!(report.forms[&a].is_a, vec![b]);
    }

    #[test]
    fn a_property_chain_makes_a_more_general_attribute_redundant() {
        // spec/14 Rule 2, the reference implementation's second pass.
        // Given `findingSite ∘ partOf ⊑ findingSite`, a concept stating
        // both `findingSite = Hand` and `findingSite = UpperLimb` needs
        // only the first: Hand is part of UpperLimb, so the chain already
        // entails the second. Rule 1 cannot see this — neither attribute
        // subsumes the other by role hierarchy and concept subsumption.
        let finding_site = id(1200);
        let part_of = id(1201);
        let hand = id(1202);
        let upper_limb = id(1203);
        let disorder = id(1204);
        let hand_disorder = id(1205);

        let axioms = vec![
            ax(&format!(
                "SubObjectPropertyOf(ObjectPropertyChain(:{finding_site} :{part_of}) :{finding_site})"
            )),
            // Hand is part of Upper limb — the edge the node graph needs,
            // and it must be in Hand's own normal form to be seen.
            ax(&format!(
                "SubClassOf(:{hand} ObjectSomeValuesFrom(:{part_of} :{upper_limb}))"
            )),
            ax(&format!(
                "EquivalentClasses(:{hand_disorder} ObjectIntersectionOf(:{disorder} \
                 ObjectSomeValuesFrom(:{finding_site} :{hand}) \
                 ObjectSomeValuesFrom(:{finding_site} :{upper_limb})))"
            )),
        ];

        let report = necessary_normal_form(&axioms);
        let form = &report.forms[&hand_disorder];
        let sites: Vec<SctId> = form
            .attributes
            .iter()
            .filter(|a| a.type_id == finding_site)
            .map(|a| a.destination_id)
            .collect();
        assert_eq!(
            sites,
            vec![hand],
            "the Upper limb site is implied by the Hand site through the chain"
        );
    }

    #[test]
    fn a_transitive_property_makes_a_reachable_attribute_redundant() {
        // A `TransitiveObjectProperty` is the chain `r ∘ r ⊑ r`, so it
        // needs no separate rule: stating both `partOf = Hand` and
        // `partOf = UpperLimb` when Hand is part of Upper limb keeps only
        // the more specific one.
        let part_of = id(1210);
        let hand = id(1211);
        let upper_limb = id(1212);
        let structure = id(1213);
        let thing = id(1214);

        let axioms = vec![
            ax(&format!("TransitiveObjectProperty(:{part_of})")),
            ax(&format!(
                "SubClassOf(:{hand} ObjectSomeValuesFrom(:{part_of} :{upper_limb}))"
            )),
            ax(&format!(
                "EquivalentClasses(:{thing} ObjectIntersectionOf(:{structure} \
                 ObjectSomeValuesFrom(:{part_of} :{hand}) \
                 ObjectSomeValuesFrom(:{part_of} :{upper_limb})))"
            )),
        ];

        let report = necessary_normal_form(&axioms);
        let parts: Vec<SctId> = report.forms[&thing]
            .attributes
            .iter()
            .filter(|a| a.type_id == part_of)
            .map(|a| a.destination_id)
            .collect();
        assert_eq!(parts, vec![hand]);
    }

    #[test]
    fn a_chain_does_not_eliminate_an_unreachable_attribute() {
        // The guard against over-elimination: same chain, but Hand is not
        // part of Foot, so both sites survive. Rule 2 must fire on
        // reachability, not on the chain's mere existence.
        let finding_site = id(1220);
        let part_of = id(1221);
        let hand = id(1222);
        let foot = id(1223);
        let disorder = id(1224);
        let odd_disorder = id(1225);

        let axioms = vec![
            ax(&format!(
                "SubObjectPropertyOf(ObjectPropertyChain(:{finding_site} :{part_of}) :{finding_site})"
            )),
            ax(&format!(
                "EquivalentClasses(:{odd_disorder} ObjectIntersectionOf(:{disorder} \
                 ObjectSomeValuesFrom(:{finding_site} :{hand}) \
                 ObjectSomeValuesFrom(:{finding_site} :{foot})))"
            )),
        ];

        let mut sites: Vec<SctId> = necessary_normal_form(&axioms).forms[&odd_disorder]
            .attributes
            .iter()
            .filter(|a| a.type_id == finding_site)
            .map(|a| a.destination_id)
            .collect();
        sites.sort();
        let mut expected = vec![hand, foot];
        expected.sort();
        assert_eq!(sites, expected, "neither site implies the other");
    }
}
