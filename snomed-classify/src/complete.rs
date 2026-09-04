//! The worklist completion (saturation) algorithm — rules CR1–CR5 of
//! `spec/13-classification.md` — run over a [`NormalizedTBox`].

use std::collections::{HashMap, HashSet, VecDeque};

use crate::normalize::NormalizedTBox;
use crate::types::{ConceptId, RoleId};

/// Precomputed lookup indices over a [`NormalizedTBox`], so each worklist
/// event only touches the rules it could possibly trigger.
struct Indices {
    /// NF1 rules, referenced by index from `by_conjunct`.
    nf1_rules: Vec<(Vec<ConceptId>, ConceptId)>,
    /// concept -> indices of NF1 rules where it's one of the conjuncts.
    nf1_by_conjunct: HashMap<ConceptId, Vec<usize>>,
    /// `A -> [(r, B)]` for NF2 rules `A ⊑ ∃r.B`.
    nf2_by_source: HashMap<ConceptId, Vec<(RoleId, ConceptId)>>,
    /// `(r, A) -> [B]` for NF3 rules `∃r.A ⊑ B`.
    nf3_by_role_filler: HashMap<(RoleId, ConceptId), Vec<ConceptId>>,
    /// `r -> [s]` for role hierarchy `r ⊑ s`.
    role_hierarchy: HashMap<RoleId, Vec<RoleId>>,
    /// `(r, s) -> [t]` for role composition `r ∘ s ⊑ t`.
    role_composition: HashMap<(RoleId, RoleId), Vec<RoleId>>,
}

impl Indices {
    fn build(tbox: &NormalizedTBox) -> Self {
        let mut nf1_by_conjunct: HashMap<ConceptId, Vec<usize>> = HashMap::new();
        for (i, (conjuncts, _)) in tbox.nf1.iter().enumerate() {
            for &c in conjuncts {
                nf1_by_conjunct.entry(c).or_default().push(i);
            }
        }
        let mut nf2_by_source: HashMap<ConceptId, Vec<(RoleId, ConceptId)>> = HashMap::new();
        for &(a, r, b) in &tbox.nf2 {
            nf2_by_source.entry(a).or_default().push((r, b));
        }
        let mut nf3_by_role_filler: HashMap<(RoleId, ConceptId), Vec<ConceptId>> = HashMap::new();
        for &(r, a, b) in &tbox.nf3 {
            nf3_by_role_filler.entry((r, a)).or_default().push(b);
        }
        let mut role_hierarchy: HashMap<RoleId, Vec<RoleId>> = HashMap::new();
        for &(r, s) in &tbox.role_hierarchy {
            role_hierarchy.entry(r).or_default().push(s);
        }
        let mut role_composition: HashMap<(RoleId, RoleId), Vec<RoleId>> = HashMap::new();
        for &(r, s, t) in &tbox.role_composition {
            role_composition.entry((r, s)).or_default().push(t);
        }
        Indices {
            nf1_rules: tbox.nf1.clone(),
            nf1_by_conjunct,
            nf2_by_source,
            nf3_by_role_filler,
            role_hierarchy,
            role_composition,
        }
    }
}

/// The saturated `S`/`R` sets: `S(X)` (subsumers, including `X` itself)
/// and `R(r)` (role successor pairs), indexed both by source and target
/// for CR3/CR4's bidirectional lookups.
pub(crate) struct CompletionState {
    pub(crate) subsumers: HashMap<ConceptId, HashSet<ConceptId>>,
    successors: HashMap<ConceptId, Vec<(RoleId, ConceptId)>>,
    predecessors: HashMap<ConceptId, Vec<(RoleId, ConceptId)>>,
}

enum Event {
    Subsumer(ConceptId, ConceptId),
    RolePair(RoleId, ConceptId, ConceptId),
}

pub(crate) fn saturate(tbox: &NormalizedTBox) -> CompletionState {
    let indices = Indices::build(tbox);
    let mut state = CompletionState {
        subsumers: HashMap::new(),
        successors: HashMap::new(),
        predecessors: HashMap::new(),
    };
    let mut role_pairs_seen: HashSet<(RoleId, ConceptId, ConceptId)> = HashSet::new();
    let mut queue: VecDeque<Event> = VecDeque::new();

    for &c in &tbox.all_concepts {
        if state.subsumers.entry(c).or_default().insert(c) {
            queue.push_back(Event::Subsumer(c, c));
        }
    }

    // Every branch below follows the same two-phase shape: scan the
    // relevant *borrowed* state/indices to collect the (typically small)
    // list of deltas this event produces, THEN apply them with `state`
    // borrowed mutably. This matters far beyond satisfying the borrow
    // checker: the naive alternative — `.cloned()`-ing the whole
    // `S(Y)`/successor/predecessor collection up front so the mutable
    // borrow is free to start immediately — clones a collection whose
    // size scales with how many facts have already been derived, on
    // *every* event that touches it. For a concept with thousands of
    // accumulated subsumers, that turns one O(1)-ish event into an
    // O(n) copy, repeated for every event that touches it — exactly the
    // kind of accidental quadratic blowup that made an early,
    // `.cloned()`-based version of this loop take minutes on a
    // synthetic 20k-concept ontology instead of the sub-second runtime
    // the algorithm's real (near-linear at SNOMED's actual hierarchy
    // shape) complexity promises. Don't reintroduce `.cloned()` here.
    while let Some(event) = queue.pop_front() {
        match event {
            Event::Subsumer(x, a) => {
                // CR1
                if let Some(rule_idxs) = indices.nf1_by_conjunct.get(&a) {
                    let sx = &state.subsumers[&x];
                    let newly_entailed: Vec<ConceptId> = rule_idxs
                        .iter()
                        .filter_map(|&ri| {
                            let (conjuncts, b) = &indices.nf1_rules[ri];
                            conjuncts.iter().all(|c| sx.contains(c)).then_some(*b)
                        })
                        .collect();
                    for b in newly_entailed {
                        add_subsumer(&mut state, &mut queue, x, b);
                    }
                }
                // CR2
                if let Some(list) = indices.nf2_by_source.get(&a) {
                    for &(r, b) in list {
                        add_role_pair(&mut state, &mut queue, &mut role_pairs_seen, r, x, b);
                    }
                }
                // CR3 (triggered by an S(Y) change, where X is the
                // predecessor: (w,x) ∈ R(r) for some w, r).
                if let Some(preds) = state.predecessors.get(&x) {
                    let newly_entailed: Vec<(ConceptId, ConceptId)> = preds
                        .iter()
                        .filter_map(|&(r, w)| {
                            indices.nf3_by_role_filler.get(&(r, a)).map(|cs| (w, cs))
                        })
                        .flat_map(|(w, cs)| cs.iter().map(move |&c| (w, c)))
                        .collect();
                    for (w, c) in newly_entailed {
                        add_subsumer(&mut state, &mut queue, w, c);
                    }
                }
            }
            Event::RolePair(r, x, y) => {
                // CR3 (triggered by an R(r) change).
                if let Some(sy) = state.subsumers.get(&y) {
                    let newly_entailed: Vec<ConceptId> = sy
                        .iter()
                        .filter_map(|a| indices.nf3_by_role_filler.get(&(r, *a)))
                        .flatten()
                        .copied()
                        .collect();
                    for c in newly_entailed {
                        add_subsumer(&mut state, &mut queue, x, c);
                    }
                }
                // CR5: role hierarchy.
                if let Some(supers) = indices.role_hierarchy.get(&r) {
                    for &s in supers {
                        add_role_pair(&mut state, &mut queue, &mut role_pairs_seen, s, x, y);
                    }
                }
                // CR4 forward: (x,y)∈R(r), (y,z)∈R(s), r∘s⊑t ⟹ (x,z)∈R(t).
                if let Some(succs) = state.successors.get(&y) {
                    let newly_entailed: Vec<(RoleId, ConceptId)> = succs
                        .iter()
                        .filter_map(|&(s, z)| {
                            indices
                                .role_composition
                                .get(&(r, s))
                                .map(|targets| (targets, z))
                        })
                        .flat_map(|(targets, z)| targets.iter().map(move |&t| (t, z)))
                        .collect();
                    for (t, z) in newly_entailed {
                        add_role_pair(&mut state, &mut queue, &mut role_pairs_seen, t, x, z);
                    }
                }
                // CR4 backward: (w,x)∈R(r1), (x,y)∈R(r), r1∘r⊑t ⟹ (w,y)∈R(t).
                if let Some(preds) = state.predecessors.get(&x) {
                    let newly_entailed: Vec<(RoleId, ConceptId)> = preds
                        .iter()
                        .filter_map(|&(r1, w)| {
                            indices
                                .role_composition
                                .get(&(r1, r))
                                .map(|targets| (targets, w))
                        })
                        .flat_map(|(targets, w)| targets.iter().map(move |&t| (t, w)))
                        .collect();
                    for (t, w) in newly_entailed {
                        add_role_pair(&mut state, &mut queue, &mut role_pairs_seen, t, w, y);
                    }
                }
            }
        }
    }

    state
}

fn add_subsumer(
    state: &mut CompletionState,
    queue: &mut VecDeque<Event>,
    x: ConceptId,
    a: ConceptId,
) {
    if state.subsumers.entry(x).or_default().insert(a) {
        queue.push_back(Event::Subsumer(x, a));
    }
}

fn add_role_pair(
    state: &mut CompletionState,
    queue: &mut VecDeque<Event>,
    seen: &mut HashSet<(RoleId, ConceptId, ConceptId)>,
    r: RoleId,
    x: ConceptId,
    y: ConceptId,
) {
    if seen.insert((r, x, y)) {
        state.successors.entry(x).or_default().push((r, y));
        state.predecessors.entry(y).or_default().push((r, x));
        queue.push_back(Event::RolePair(r, x, y));
    }
}
