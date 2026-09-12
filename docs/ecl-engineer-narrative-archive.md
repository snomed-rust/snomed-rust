# ECL engineer narrative archive

The `{{ }}` filters increment-by-increment narrative in
[`agents/ecl-engineer.md`](../agents/ecl-engineer.md) grew past that
file's own 40 KB budget (`spec/docs-budget-and-links/index.md` rule 1).
This is the oldest portion — `{{ C }}`/`{{ D }}`'s own early history,
`{{ M }}`'s first three shared-column kinds, and `memberFieldFilter`'s
first nineteen columns (`mapTarget` through `guideURL`, completing
`MrcmDomainRefsetMember`'s column coverage on 2026-09-08) — moved
here verbatim so `agents/ecl-engineer.md` can keep going. Newer
entries live there; this is a historical record, not a summary, and
worth searching before assuming a lesson is new.

The official grammar's `{{ }}` filter subsystem is large — description,
concept, and member filter constraints, each with several filter kinds
(`term`/`language`/`type`/`dialect`/`module`/`effectiveTime`/`active`/
`definitionStatus`/refset field), most needing their own value-set
grammar. Implementing all of it in one increment isn't realistic, and it
hasn't been: `{{ C ... }}` came first (`activeFilter`, then
`definitionStatusTokenFilter`, then `moduleFilter`, then
`effectiveTimeFilter`, then `definitionStatusIdFilter` — one commit
each), and `{{ D ... }}` followed the same way (`term` with its search
types, `type`/`typeId`, `language`, `dialectId` with its acceptability
set, `moduleId`, `effectiveTime`, `active`). `{{ M ... }}` picked up the
same three kinds in one increment (`moduleId`, `effectiveTime`, `active`
— they share `ModuleFilter`/`EffectiveTimeFilter`/`ActiveFilter` with
`{{ C }}`, so there was no new value-set grammar to design), after the
`SnapshotStore` change spec/09 rule 4's `member_rows`/`member_components`
index required first. `{{ M ... }}` after `^R` followed on 2026-09-02,
needing its own store index (`member_refsets`/`all_member_concepts` —
the inactive-inclusive reverse of `refsets_containing`, spec/09 rule 4)
since `^R`'s row-per-candidate shape can't reuse `^`'s. Its
refset-type-specific `memberFieldFilter` kind was rejected only
generically until 2026-09-03: reaching a type-specific column needed its
*own* store-retention decision first (`plan.md`'s "Open decisions",
decided 2026-09-03 — all sixteen non-Simple/Language types), the same
shape of call `member_rows`/`member_refsets` both needed. `mapTarget`
landed the same day as that decision's first concrete field, tested
against `SnapshotStore::simple_map_member_rows`/
`extended_map_member_rows` rather than `member_rows`'s type-erased view;
`correlationId`, `mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, and
`mapCategoryId` followed immediately after, one to three fields each of
two more `memberFieldFilter` grammar shapes — `mapCategoryId` reusing
`correlationId`'s exact shape and completing
`ExtendedMapRefsetMember`'s column coverage. `targetComponentId`
(2026-09-05) is the first column outside the two map types
(`AssociationRefsetMember`), reusing `correlationId`'s concept-reference
shape again but tested against a third typed row set
(`association_member_rows`) — the dispatch function that used to be
`typed_map_row_matches` is `typed_field_row_matches` now that it isn't
map-only. `valueId` (2026-09-06) is the second column outside the two
map types (`AttributeValueRefsetMember`), the same shape again but a
fourth row-set check (`attribute_value_member_rows`) added to the same
function; `owlExpression` (2026-09-06) is the third, on the
string-search shape this time (`OwlExpressionRefsetMember`, a fifth
row-set check) rather than concept-reference; `order` (2026-09-06) is
the fourth, back on the numeric shape (`OrderedComponentRefsetMember`,
a sixth row-set check, reusing `mapGroup`/`mapPriority`'s
`field_numeric_matches` verbatim) — four increments in a
row confirming the pattern generalizes cleanly across every grammar
shape: extend `TypedFields`, extend the dispatch condition, add one
row-set check to `typed_field_row_matches`, done. Then
`OrderedAssociationRefsetMember` (2026-09-06) — a fifth type outside
the two map types, carrying *both* `targetComponentId` and `order` on
one row — needed no new variant at all: a seventh row-set check
(`ordered_association_member_rows`) populating both existing
`TypedFields` entries from the same row, confirming the "reuse the
existing variant" path the doc comments on both `TargetComponentId`
and `Order` had already flagged. `mrcmRuleRefsetId`
(`MrcmModuleScopeRefsetMember`, 2026-09-06) broke that streak
deliberately: it's the sixth type outside the two map types, but the
first since `targetComponentId` where no implemented column shares the
RF2 field name, so it needed a genuinely new variant
(`MrcmRuleRefsetId`) and an eighth row-set check
(`mrcm_module_scope_member_rows`) — the "reuse" shortcut only fires
when the column *name* recurs, not just the shape.
`attributeDescription` (`RefsetDescriptorRefsetMember`, 2026-09-06)
followed the same non-reuse path: the seventh type outside the two map
types, another genuinely new variant (`AttributeDescription`) since no
implemented column shares its RF2 field name either, and a ninth
row-set check (`refset_descriptor_member_rows`) — the store already
carried that accessor (`RefsetDescriptorRefsetMember` was already one
of the sixteen retained types), so this increment was parser/eval only,
no `snomed-store` change. `attributeType` (2026-09-06),
`RefsetDescriptorRefsetMember`'s second column, followed immediately:
another genuinely new variant (`AttributeType`) since no implemented
column shares that RF2 field name either, but this one needed no new
row-set check at all — both columns live on the same row, so
`refset_descriptor_member_rows`' existing block just grew a second
`TypedFields` entry populated alongside the first, the same "two
fields, one row" shape `targetComponentId`/`order` already have for
`OrderedAssociationRefsetMember`. `attributeOrder` (2026-09-06),
`RefsetDescriptorRefsetMember`'s third and last column, followed the
same pattern once more: back on the numeric shape (reusing
`mapGroup`/`mapPriority`/`order`'s `field_numeric_matches`), another
genuinely new variant (`AttributeOrder`, since no implemented column
shares that field name), and again no new row-set check — the same
`refset_descriptor_member_rows` block now populates all three
`TypedFields` entries from one row. `descriptionFormat` (2026-09-07),
`DescriptionTypeRefsetMember`'s first column, is the eighth type
outside the two map types: back on the concept-reference shape, another
genuinely new variant (`DescriptionFormat`, since no implemented column
shares that field name), and this time a genuinely new tenth row-set
check (`description_type_member_rows`) — the store already carried that
accessor too, so again a parser/eval-only increment. `descriptionLength`
(2026-09-07), `DescriptionTypeRefsetMember`'s second and last column,
followed immediately: back on the numeric shape (reusing
`mapGroup`/`mapPriority`/`order`/`attributeOrder`'s
`field_numeric_matches`), another genuinely new variant
(`DescriptionLength`, since no implemented column shares that field
name), but this time no new row-set check — both of that type's
columns share one row, the same "two fields, one row" shape
`AttributeType`/`AttributeOrder` have on `RefsetDescriptorRefsetMember`.
`domainConstraint` (2026-09-07), `MrcmDomainRefsetMember`'s first
column, followed: the ninth type outside the two map types, and the
first of those nine on the string-search shape — reuses
`mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`'s `term_matches`,
another genuinely new variant (`DomainConstraint`), and a genuinely
new eleventh row-set check (`mrcm_domain_member_rows`) since it's that
type's first filterable column. `parentDomain` (2026-09-07),
`MrcmDomainRefsetMember`'s second column, followed immediately: another
genuinely new variant (`ParentDomain`), but no new row-set check —
`domainConstraint`/`parentDomain` share one row, the same reuse
`attributeDescription`/`attributeType` established on
`RefsetDescriptorRefsetMember`. `proximalPrimitiveConstraint`
(2026-09-07), `MrcmDomainRefsetMember`'s third column, followed
immediately too: another genuinely new variant
(`ProximalPrimitiveConstraint`), again no new row-set check — all
three of that type's columns now share one row.
`proximalPrimitiveRefinement` (2026-09-07),
`MrcmDomainRefsetMember`'s fourth column, followed immediately too:
another genuinely new variant (`ProximalPrimitiveRefinement`), again
no new row-set check — all four of that type's columns now share one
row. `domainTemplateForPrecoordination` (2026-09-08),
`MrcmDomainRefsetMember`'s fifth column, followed immediately too:
another genuinely new variant (`DomainTemplateForPrecoordination`),
again no new row-set check — all five of that type's columns now
share one row. `domainTemplateForPostcoordination` (2026-09-08),
`MrcmDomainRefsetMember`'s sixth column, followed immediately too:
another genuinely new variant (`DomainTemplateForPostcoordination`),
again no new row-set check — all six of that type's columns now
share one row. `guideURL` (2026-09-08),
`MrcmDomainRefsetMember`'s seventh and last column, followed
immediately too: another genuinely new variant (`GuideUrl`), again no
new row-set check — all seven of that type's columns now share one
row, completing its column coverage, the third refset type outside
the two map types (after `RefsetDescriptorRefsetMember` and
`DescriptionTypeRefsetMember`) to reach it.