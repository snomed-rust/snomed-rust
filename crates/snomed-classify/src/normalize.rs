//! Structural transformation from `snomed_owl::Axiom`s into the EL normal
//! forms (NF1–NF3, role hierarchy, role composition) `spec/13-classification.md`
//! defines.

use snomed_core::sctid::SctId;
use snomed_owl::{Axiom, ClassExpression, ObjectPropertyExpression};

use crate::skipped::SkippedConstruct;
use crate::types::{ConceptId, RoleId};

/// The normalized TBox, plus every concept/role id (named or fresh) that
/// needs a seeded entry in the completion state.
#[derive(Debug, Default)]
pub(crate) struct NormalizedTBox {
    /// `A1 ⊓ ... ⊓ An ⊑ B` rules (NF1).
    pub(crate) nf1: Vec<(Vec<ConceptId>, ConceptId)>,
    /// `A ⊑ ∃r.B` rules (NF2), as `(A, r, B)`.
    pub(crate) nf2: Vec<(ConceptId, RoleId, ConceptId)>,
    /// `∃r.A ⊑ B` rules (NF3), as `(r, A, B)`.
    pub(crate) nf3: Vec<(RoleId, ConceptId, ConceptId)>,
    /// `r ⊑ s` role hierarchy axioms.
    pub(crate) role_hierarchy: Vec<(RoleId, RoleId)>,
    /// `r ∘ s ⊑ t` role composition axioms.
    pub(crate) role_composition: Vec<(RoleId, RoleId, RoleId)>,
    /// Every concept id (named or fresh) that appeared anywhere — the
    /// seed set for completion.
    pub(crate) all_concepts: std::collections::HashSet<ConceptId>,
    pub(crate) skipped: Vec<SkippedConstruct>,
}

struct Normalizer {
    tbox: NormalizedTBox,
    next_fresh_concept: u32,
    next_fresh_role: u32,
}

pub(crate) fn normalize<'a>(axioms: impl IntoIterator<Item = &'a Axiom>) -> NormalizedTBox {
    let mut n = Normalizer {
        tbox: NormalizedTBox::default(),
        next_fresh_concept: 0,
        next_fresh_role: 0,
    };
    for axiom in axioms {
        n.add_axiom(axiom);
    }
    n.tbox
}

impl Normalizer {
    fn fresh_concept(&mut self) -> ConceptId {
        let id = ConceptId::Fresh(self.next_fresh_concept);
        self.next_fresh_concept += 1;
        self.seed(id);
        id
    }

    fn fresh_role(&mut self) -> RoleId {
        let id = RoleId::Fresh(self.next_fresh_role);
        self.next_fresh_role += 1;
        id
    }

    fn seed(&mut self, id: ConceptId) {
        self.tbox.all_concepts.insert(id);
    }

    fn named(&mut self, id: SctId) -> ConceptId {
        let c = ConceptId::Named(id);
        self.seed(c);
        c
    }

    fn add_axiom(&mut self, axiom: &Axiom) {
        match axiom {
            Axiom::SubClassOf { sub, sup } => self.add_subclass_of(sub, sup),
            Axiom::EquivalentClasses(ops) => {
                // A cycle of pairwise SubClassOf axioms is sufficient for
                // full mutual subsumption — the completion algorithm's
                // own transitive rule-firing does the rest (spec/13).
                for i in 0..ops.len() {
                    let j = (i + 1) % ops.len();
                    self.add_subclass_of(&ops[i], &ops[j]);
                }
            }
            Axiom::SubObjectPropertyOf { sub, sup } => {
                let target = RoleId::Named(*sup);
                match sub {
                    ObjectPropertyExpression::Named(r) => {
                        self.tbox.role_hierarchy.push((RoleId::Named(*r), target));
                    }
                    ObjectPropertyExpression::Chain(ids) => self.add_role_chain(ids, target),
                }
            }
            Axiom::SubDataPropertyOf { sub, .. } => {
                self.tbox.skipped.push(SkippedConstruct::DataProperty(*sub));
            }
            Axiom::TransitiveObjectProperty(r) => {
                let rid = RoleId::Named(*r);
                self.tbox.role_composition.push((rid, rid, rid));
            }
            Axiom::ReflexiveObjectProperty(r) => {
                self.tbox
                    .skipped
                    .push(SkippedConstruct::ReflexiveProperty(*r));
            }
        }
    }

    /// Folds an `ObjectPropertyChain` of any length into binary role
    /// compositions, introducing fresh intermediate roles as needed
    /// (length 2 — SNOMED's only observed real usage, spec/12 — needs
    /// none).
    fn add_role_chain(&mut self, ids: &[SctId], target: RoleId) {
        debug_assert!(ids.len() >= 2, "ObjectPropertyChain has 2+ operands");
        let mut acc = RoleId::Named(ids[0]);
        for &next in &ids[1..ids.len() - 1] {
            let fresh = self.fresh_role();
            self.tbox
                .role_composition
                .push((acc, RoleId::Named(next), fresh));
            acc = fresh;
        }
        let last = RoleId::Named(*ids.last().expect("2+ operands"));
        self.tbox.role_composition.push((acc, last, target));
    }

    fn add_subclass_of(&mut self, sub: &ClassExpression, sup: &ClassExpression) {
        let conjuncts = self.normalize_conjuncts(sub);
        if conjuncts.is_empty() {
            return; // sub contributed nothing classifiable (spec/13 scope)
        }
        self.emit_subsumption(&conjuncts, sup);
    }

    /// Flattens a top-level `ObjectIntersectionOf` into its conjunct
    /// concept ids (recursively normalizing each operand); any other
    /// expression normalizes to a single-element list.
    fn normalize_conjuncts(&mut self, expr: &ClassExpression) -> Vec<ConceptId> {
        match expr {
            ClassExpression::ObjectIntersectionOf(ops) => ops
                .iter()
                .filter_map(|op| self.normalize_conjunct_operand(op))
                .collect(),
            other => match self.normalize_conjunct_operand(other) {
                Some(id) => vec![id],
                None => vec![],
            },
        }
    }

    /// `None` for a `DataHasValue` operand (dropped per spec/13 scope,
    /// and reported in `skipped`); otherwise the operand's concept id.
    fn normalize_conjunct_operand(&mut self, expr: &ClassExpression) -> Option<ConceptId> {
        match expr {
            ClassExpression::DataHasValue { attribute, .. } => {
                self.tbox.skipped.push(SkippedConstruct::ConcreteValue {
                    attribute: *attribute,
                });
                None
            }
            other => Some(self.normalize_concept_to_id(other)),
        }
    }

    /// Converts any class expression into a single concept id, defining
    /// a fresh name equivalent to it (in both directions) when it isn't
    /// already a plain concept reference.
    fn normalize_concept_to_id(&mut self, expr: &ClassExpression) -> ConceptId {
        match expr {
            ClassExpression::Concept(id) => self.named(*id),
            ClassExpression::ObjectIntersectionOf(ops) => {
                let conjuncts: Vec<ConceptId> = ops
                    .iter()
                    .filter_map(|op| self.normalize_conjunct_operand(op))
                    .collect();
                match conjuncts.as_slice() {
                    [] => self.fresh_concept(), // every operand was DataHasValue
                    [single] => *single,
                    _ => {
                        let fresh = self.fresh_concept();
                        self.tbox.nf1.push((conjuncts.clone(), fresh)); // conjunction ⊑ F
                        for c in conjuncts {
                            self.tbox.nf1.push((vec![fresh], c)); // F ⊑ each conjunct
                        }
                        fresh
                    }
                }
            }
            ClassExpression::ObjectSomeValuesFrom { attribute, filler } => {
                let filler_id = self.normalize_concept_to_id(filler);
                let fresh = self.fresh_concept();
                let role = RoleId::Named(*attribute);
                self.tbox.nf2.push((fresh, role, filler_id)); // F ⊑ ∃r.D
                self.tbox.nf3.push((role, filler_id, fresh)); // ∃r.D ⊑ F
                fresh
            }
            ClassExpression::DataHasValue { attribute, .. } => {
                self.tbox.skipped.push(SkippedConstruct::ConcreteValue {
                    attribute: *attribute,
                });
                self.fresh_concept() // isolated atom: matches/is matched by nothing
            }
        }
    }

    /// Distributes `sup` (a class expression appearing on the right of a
    /// `SubClassOf`, or reached recursively while distributing a
    /// top-level intersection) into NF1/NF2 rules keyed by `conjuncts`.
    fn emit_subsumption(&mut self, conjuncts: &[ConceptId], sup: &ClassExpression) {
        match sup {
            ClassExpression::Concept(id) => {
                let target = self.named(*id);
                self.tbox.nf1.push((conjuncts.to_vec(), target));
            }
            ClassExpression::ObjectIntersectionOf(ops) => {
                for op in ops {
                    self.emit_subsumption(conjuncts, op);
                }
            }
            ClassExpression::ObjectSomeValuesFrom { attribute, filler } => {
                let filler_id = self.normalize_concept_to_id(filler);
                let role = RoleId::Named(*attribute);
                let source = match conjuncts {
                    [single] => *single,
                    _ => {
                        // Multiple conjuncts on the left, existential on
                        // the right: NF2 needs a single source concept,
                        // so introduce one standing for "all conjuncts
                        // hold". Only the conjunction -> fresh direction
                        // is needed here (nothing needs to derive the
                        // conjuncts back out of this particular fresh
                        // name).
                        let fresh = self.fresh_concept();
                        self.tbox.nf1.push((conjuncts.to_vec(), fresh));
                        fresh
                    }
                };
                self.tbox.nf2.push((source, role, filler_id));
            }
            ClassExpression::DataHasValue { attribute, .. } => {
                self.tbox.skipped.push(SkippedConstruct::ConcreteValue {
                    attribute: *attribute,
                });
            }
        }
    }
}
