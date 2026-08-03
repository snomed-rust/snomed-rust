//! SNOMED CT for Rust — facade crate.
//!
//! Re-exports the workspace crates under one roof:
//!
//! - [`core`] ([`snomed_core`]) — SCTIDs, components, constants;
//! - [`rf2`] ([`snomed_rf2`]) — RF2 release file parsing;
//! - [`store`] ([`snomed_store`]) — snapshot store and hierarchy queries;
//! - [`ecl`] ([`snomed_ecl`]) — Expression Constraint Language (simple
//!   constraints subset).
//!
//! ```
//! use snomed::prelude::*;
//!
//! let concepts = "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
//!     138875005\t20190731\t1\t900000000000207008\t900000000000074008\n";
//! let mut builder = SnapshotStore::builder();
//! builder.add_concepts(read_all::<_, Concept>(concepts.as_bytes()).unwrap());
//! let store = builder.build();
//! assert!(store.is_active(constants::ROOT_CONCEPT));
//! ```

pub use snomed_core as core;
pub use snomed_ecl as ecl;
pub use snomed_rf2 as rf2;
pub use snomed_store as store;

/// The commonly needed names in one import.
pub mod prelude {
    pub use snomed_core::components::{
        Concept, Description, Relationship, RelationshipConcreteValue,
    };
    pub use snomed_core::concrete_value::{ConcreteValue, ConcreteValueError};
    pub use snomed_core::constants;
    pub use snomed_core::sctid::{ComponentType, SctId, SctIdError};
    pub use snomed_core::time::EffectiveTime;
    pub use snomed_rf2::filename::ReleaseFileName;
    pub use snomed_rf2::reader::{read_all, Rf2Reader};
    pub use snomed_rf2::refset::{
        AssociationRefsetMember, AttributeValueRefsetMember, DescriptionTypeRefsetMember,
        ExtendedMapRefsetMember, LanguageRefsetMember, ModuleDependencyRefsetMember,
        OwlExpressionRefsetMember, RefsetDescriptorRefsetMember, SimpleMapRefsetMember,
        SimpleRefsetMember,
    };
    pub use snomed_rf2::release_type::ReleaseType;
    pub use snomed_store::{LoadError, LoadReport, SnapshotStore, SnapshotStoreBuilder};

    pub use snomed_ecl::{
        evaluate as evaluate_ecl, parse as parse_ecl, AttributeConstraint, EclError,
        ExpressionConstraint, FocusConcept, HierarchyOp, RefinementConstraint,
        SimpleExpressionConstraint,
    };
}
