//! The completeness verdict: what the reader's privileges did not reach
//! (`FR-PRIV-001` … `FR-PRIV-007`, `FR-PRIV-011` … `FR-PRIV-019`).
//!
//! A reader whose privileges fall short of the model is **not refused by the
//! server**. It receives rows, and the shortfall is inside them. This module
//! holds the three checks that find it, the marking they build, and the
//! verdict a caller that named one object turns into `77`.
//!
//! # Three shapes, and therefore three checks
//!
//! `FR-PRIV-018` records that a privilege-driven absence reaches a reader in
//! three shapes that do not resemble one another, observed on all four series
//! of `FR-SRV-015`:
//!
//! | Shape | The property it hides | Check | Fixed by |
//! |---|---|---|---|
//! | The **empty string** | A view's definition | [`empty_string`] | `FR-PRIV-011` |
//! | SQL **`NULL`** | A routine's body | [`sql_null`] | `FR-PRIV-017` |
//! | **Zero rows** | A table's referential rules | [`no_row`] | `FR-PRIV-019` |
//!
//! They are three functions rather than one emptiness test, because one test
//! finds one shape and misses the other two: an empty string is not a `NULL`,
//! and neither of them is a row that is not there. Each takes the value in the
//! shape the catalogue gave it — a `&str` for the first, an [`Option`] for the
//! second, and the answer of the rules lookup for the third — so no check can
//! be handed a value another one has already flattened.
//!
//! *There is no fourth check.* A table's triggers reach a reader without the
//! privilege as zero rows and reach a reader of a table that has none as zero
//! rows, and `FR-PRIV-020` states in terms that `tpl` SHALL NOT claim to
//! distinguish them. `triggers` therefore never appears in a marking.
//!
//! # Where each check is made, and on which object kinds
//!
//! Every check is made **in the fold, as the rows are mapped**, and the fold is
//! the one path every read takes — schema-wide, narrowed to one object, and the
//! dump alike. `FR-PRIV-012` is satisfied by that construction rather than by a
//! call site per command, and there is no second pass over the finished model
//! for a later read to forget.
//!
//! `FR-PRIV-015` closes the set of object kinds a **cross-check** is made for,
//! and this module observes it: the view's, through [`empty_string`], and the
//! foreign key's, through [`no_row`]. A routine's body needs no cross-check at
//! all, because SQL `NULL` announces itself; nothing cross-checks a table's
//! columns, indexes, check constraints or triggers, or a routine's parameters,
//! because the catalogue offers no second observation of any of them.
//!
//! # The foreign-key shortfall marks both ends
//!
//! `FR-PRIV-019` detects one observation — a key column naming a referenced
//! table under no referential-constraint row — and that one observation costs
//! **two** tables a property, because `FR-CAT-045` presents one key from two
//! ends:
//!
//! | Table | Property it loses | Presented by |
//! |---|---|---|
//! | The referencing table, which carries the key column | `foreign_keys` | `FR-CAT-012` |
//! | The referenced table, named by that same column | `referenced_by` | `FR-CAT-013` |
//!
//! Marking only the first would leave the second reporting an empty
//! `referenced_by` as complete, at exit `0`, which `FR-PRIV-002` forbids in as
//! many words — *any property the model defines for it could not be read,
//! whatever the reason* — and which is the silent failure the rationale of
//! `FR-PRIV-019` names when it cites `FR-CAT-013` beside `FR-CAT-012`.
//!
//! # The `77` is a verdict here and an exit code elsewhere
//!
//! [`of_table`], [`of_view`] and [`of_routine`] answer `FR-PRIV-003` and
//! `FR-PRIV-004` for a caller that named one object: an incomplete object is
//! **not returned in part**, and the answer is
//! [`Error::PropertyNotReadable`], whose
//! code is `77`. This module does not emit that code and does not write a
//! message; it produces the value the caller returns, and the binary maps it
//! as it maps every other. Nothing on the path carries a credential, because
//! the value carries an object kind, an object name and a property name and
//! has no field a credential could occupy — which is `FR-PRIV-014` satisfied
//! by the shape of the error rather than by redaction.

use std::borrow::Cow;
use std::panic::Location;

use crate::error::{CatalogueObjectKind, Error};
use crate::model::document::order;
use crate::model::document::shape::TableDocument;
use crate::model::restricted::Restricted;
use crate::model::routine::Routine;
use crate::model::view::View;

/// The invariant a marking naming a property this reader cannot name violates.
const MARKED_PROPERTY: &str =
    "every property a completeness marking names is one this reader can record";

/// A property of the model a reader's privileges may fail to reach.
///
/// The set is **closed** and is the whole vocabulary of a marking this reader
/// builds. `FR-PRIV-016` makes the population of the `restricted` array the
/// property names of the model itself, so each variant carries the name the
/// document already presents that property under and invents none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Property {
    /// A routine's body, absent as SQL `NULL` (`FR-PRIV-017`).
    Body,

    /// A view's definition, absent as the empty string (`FR-PRIV-011`).
    Definition,

    /// The referential rules of the keys a table declares (`FR-PRIV-019`,
    /// `FR-CAT-012`).
    ForeignKeys,

    /// The referential rules of the keys that point **at** a table
    /// (`FR-PRIV-019`, `FR-CAT-013`).
    ReferencedBy,
}

impl Property {
    /// Every property this reader can report unreadable.
    ///
    /// It is what [`Property::from_name`] searches, so the two directions of
    /// the mapping are one list and one `match` rather than two literal lists
    /// that can drift apart. A variant added without an entry here would be
    /// caught by `every_property_answers_to_its_own_name`.
    const ALL: [Self; 4] = [
        Self::Body,
        Self::Definition,
        Self::ForeignKeys,
        Self::ReferencedBy,
    ];

    /// The name the document carries this property under (`FR-PRIV-016`).
    const fn name(self) -> &'static str {
        match self {
            Self::Body => "body",
            Self::Definition => "definition",
            Self::ForeignKeys => "foreign_keys",
            Self::ReferencedBy => "referenced_by",
        }
    }

    /// The property `name` names, or [`None`] where it names none of them.
    fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|property| property.name() == name)
    }
}

/// The properties of one object a read did not reach, before it is sealed into
/// the marking `FR-PRIV-016` fixes.
///
/// It exists because one object can lose more than one property — a table at
/// both ends of a foreign key loses `foreign_keys` and `referenced_by` from
/// the same observation — and because `FR-PRIV-016` forbids the empty array
/// that an always-present marking would need. A complete object never leaves
/// this type at all: [`Marking::sealed`] answers [`None`] for it, which is
/// `FR-PRIV-007`.
///
/// The default is the empty marking, and it allocates nothing: the ordinary
/// object is complete, and a value built for every view and every routine of
/// every read may not cost one allocation each.
#[derive(Debug, Default)]
pub(super) struct Marking {
    /// The properties recorded so far, without repetition.
    properties: Vec<Property>,
}

impl Marking {
    /// Records that `property` could not be read, once however often it is
    /// observed.
    ///
    /// The repetition is real and not defensive: the fifteen foreign keys of
    /// one shortfall are seventeen key columns, and several of them name the
    /// same pair of tables.
    pub(super) fn record(&mut self, property: Property) {
        if !self.properties.contains(&property) {
            self.properties.push(property);
        }
    }

    /// The marking these properties make, or [`None`] for a complete object.
    ///
    /// The order is **produced here** and not inherited from the order the
    /// properties were observed in: [`order::sort_by_name`] is the one
    /// comparator `NFR-DET-002` fixes, byte-wise and ascending, and it is the
    /// same call the document's own build sites make. `FR-PRIV-016` requires
    /// the array to be ordered, so a caller reading
    /// [`Restricted::properties`] of a marking this reader built reads it in
    /// that order without serialising it first.
    ///
    /// [`None`] is `FR-PRIV-007`: a complete object carries no marking, and
    /// [`Restricted::new`] is what refuses the empty array `FR-PRIV-016`
    /// forbids.
    pub(super) fn sealed(self) -> Option<Restricted<'static>> {
        let mut properties: Vec<Cow<'static, str>> = self
            .properties
            .into_iter()
            .map(|property| Cow::Borrowed(property.name()))
            .collect();

        order::sort_by_name(&mut properties);

        Restricted::new(properties)
    }
}

/// The first shape of `FR-PRIV-018`: a field that came back as the **empty
/// string** (`FR-PRIV-011`).
///
/// A view's definition is the view, and the empty string is not a value a view
/// can legitimately carry, so the observation admits one explanation: the
/// reader can see that the view exists and cannot read it.
pub(super) fn empty_string(definition: &str) -> bool {
    definition.is_empty()
}

/// The second shape of `FR-PRIV-018`: a field that came back as SQL **`NULL`**
/// (`FR-PRIV-017`).
///
/// The argument is the field's own nullity, read from the row. It is not the
/// value the model carries: [`super::row::text_or_empty`] substitutes the empty
/// string for a `NULL` the model has no shape for, and a body that is genuinely
/// empty is the empty string too — so a check made over the folded model would
/// answer the same for the two cases this requirement exists to separate.
pub(super) fn sql_null(body: Option<&str>) -> bool {
    body.is_none()
}

/// The third shape of `FR-PRIV-018`: a read that returned **zero rows**
/// (`FR-PRIV-019`).
///
/// The argument is what the lookup over the referential-constraint rows
/// answered for the constraint a key column belongs to. There is no row to
/// inspect and no field to decode — the absence *is* the observation — which is
/// why this check takes the answer of a lookup where the other two take a
/// value.
pub(super) fn no_row(rule: Option<usize>) -> bool {
    rule.is_none()
}

/// The verdict on a table a caller named (`FR-PRIV-003`, `FR-PRIV-004`).
///
/// It is taken over the **document's** table and not the model's, which is the
/// one shape every caller of it holds: a named read is served from the cache or
/// from a server, per `FR-CACHE-006`, and only one of the two ever produces a
/// model. `FR-PRIV-016` puts the marking on the object in either case, so the
/// verdict reads the same field whichever source answered.
///
/// # Errors
///
/// Returns [`Error::PropertyNotReadable`], whose code is `77`, where the table
/// is marked incomplete. The caller returns it rather than the table:
/// `FR-PRIV-004` bars a partial object from answering a request that named it.
#[track_caller]
pub(crate) fn of_table(table: &TableDocument<'_>) -> Result<(), Error> {
    verdict(
        CatalogueObjectKind::Table,
        &table.name,
        table.restricted.as_ref(),
    )
}

/// The verdict on a view a caller named (`FR-PRIV-003`, `FR-PRIV-004`).
///
/// # Errors
///
/// Returns [`Error::PropertyNotReadable`], whose code is `77`, where the view
/// is marked incomplete — which for a view is the unreadable definition of
/// `FR-PRIV-011`.
#[track_caller]
pub(crate) fn of_view(view: &View<'_>) -> Result<(), Error> {
    verdict(
        CatalogueObjectKind::View,
        &view.name,
        view.restricted.as_ref(),
    )
}

/// The verdict on a routine a caller named (`FR-PRIV-003`, `FR-PRIV-004`).
///
/// # Errors
///
/// Returns [`Error::PropertyNotReadable`], whose code is `77`, where the
/// routine is marked incomplete. `FR-PRIV-017` records that this is the
/// ordinary case for a least-privilege reader and not an edge one: the
/// privilege that exposes a routine body is not among those a read-only
/// catalogue user is usually granted, so such a reader receives `77` for every
/// routine of the database rather than a stub.
#[track_caller]
pub(crate) fn of_routine(routine: &Routine<'_>) -> Result<(), Error> {
    verdict(
        CatalogueObjectKind::Routine,
        &routine.name,
        routine.restricted.as_ref(),
    )
}

/// The verdict the three named-object entry points share.
///
/// The `cause` line of `FR-PRIV-013` states **one** property, and
/// [`Error::PropertyNotReadable`] carries one, so an object that lost two
/// reports the **byte-wise least** of them: the marking is ordered by
/// [`Marking::sealed`], so the first name it carries is that one, and the
/// message a caller receives is therefore the same on every run, per
/// `NFR-DET-002`.
///
/// # Errors
///
/// Returns [`Error::PropertyNotReadable`] for a marked object, and
/// [`Error::InternalInvariant`] — `70` — for a marking naming a property this
/// reader has no name for. The second is not a condition a caller can act on:
/// every marking the fold builds is built from [`Property`], so a name outside
/// that set means `tpl`'s own vocabulary disagrees with itself. Reporting it as
/// a complete object instead would answer exit `0` for an object known to be
/// short, which is the failure `BR-PRIV-001` calls the one this specification
/// works hardest to prevent.
#[track_caller]
fn verdict(
    kind: CatalogueObjectKind,
    object: &str,
    marking: Option<&Restricted<'_>>,
) -> Result<(), Error> {
    let location = Location::caller();
    let Some(marking) = marking else {
        // FR-PRIV-007: a complete object carries no marking, and there is
        // nothing for this verdict to refuse.
        return Ok(());
    };
    let property = marking
        .properties()
        .iter()
        .find_map(|name| Property::from_name(name))
        .ok_or(Error::InternalInvariant {
            invariant: MARKED_PROPERTY,
            location,
        })?;

    Err(Error::PropertyNotReadable {
        kind,
        object: object.to_owned(),
        property: property.name(),
    })
}

#[cfg(test)]
mod tests {
    use super::{Marking, Property, empty_string, no_row, of_routine, of_table, of_view, sql_null};
    use crate::error::Error;
    use crate::model::document::shape::TableDocument;
    use crate::model::restricted::Restricted;
    use crate::model::routine::{Routine, RoutineKind};
    use crate::model::table::TableType;
    use crate::model::view::View;
    use std::borrow::Cow;

    /// A view carrying `definition`, complete in every other field.
    fn view(definition: &'static str, restricted: Option<Restricted<'static>>) -> View<'static> {
        View {
            name: Cow::Borrowed("v_consignment_manifest"),
            definition: Cow::Borrowed(definition),
            check_option: Cow::Borrowed("NONE"),
            is_updatable: false,
            definer: Cow::Borrowed("root@localhost"),
            security_type: Cow::Borrowed("DEFINER"),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
            algorithm: Cow::Borrowed("UNDEFINED"),
            restricted,
        }
    }

    /// A routine carrying `body`, complete in every other field.
    fn routine(body: &'static str, restricted: Option<Restricted<'static>>) -> Routine<'static> {
        Routine {
            name: Cow::Borrowed("sp_book_consignment"),
            kind: RoutineKind::Procedure,
            return_type: None,
            parameters: Vec::new(),
            body: Some(Cow::Borrowed(body)),
            body_kind: Cow::Borrowed("SQL"),
            parameter_style: Cow::Borrowed("SQL"),
            is_deterministic: false,
            sql_data_access: Cow::Borrowed("MODIFIES SQL DATA"),
            security_type: Cow::Borrowed("DEFINER"),
            sql_mode: Cow::Borrowed(""),
            comment: Cow::Borrowed(""),
            definer: Cow::Borrowed("root@localhost"),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
            database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
            restricted,
        }
    }

    /// A table of the document carrying `restricted` and nothing else worth
    /// naming.
    fn table(restricted: Option<Restricted<'static>>) -> TableDocument<'static> {
        TableDocument {
            name: Cow::Borrowed("consignment"),
            table_type: TableType::Base,
            engine: None,
            collation: None,
            comment: Cow::Borrowed(""),
            columns: Cow::Owned(Vec::new()),
            indexes: Cow::Owned(Vec::new()),
            primary_key: None,
            foreign_keys: Vec::new(),
            referenced_by: Vec::new(),
            triggers: Cow::Owned(Vec::new()),
            check_constraints: Cow::Owned(Vec::new()),
            restricted,
        }
    }

    #[test]
    fn fr_priv_018_the_three_shapes_are_three_checks_and_none_answers_for_another() {
        // FR-PRIV-018: the empty string, SQL NULL and zero rows. The point of
        // the test is the cross terms — each check is fed the other two
        // shapes' evidence and does not fire.
        assert!(empty_string(""));
        assert!(!empty_string("select 1 AS `one`"));

        assert!(sql_null(None));
        // A body that is genuinely the empty string is not a missing
        // privilege, and the NULL check must not read it as one.
        assert!(!sql_null(Some("")));
        assert!(!sql_null(Some("BEGIN END")));

        assert!(no_row(None));
        assert!(!no_row(Some(0)));
    }

    #[test]
    fn fr_priv_020_no_check_exists_for_a_tables_triggers() {
        // FR-PRIV-020: zero rows from the trigger catalogue is byte-for-byte
        // what a table with no triggers returns, so `triggers` is a name this
        // reader's vocabulary does not contain and can never emit.
        assert_eq!(Property::from_name("triggers"), None);
    }

    #[test]
    fn fr_priv_016_every_property_answers_to_its_own_name() {
        // The two directions of the mapping agree, and the four names are the
        // model's own field names rather than a second vocabulary.
        for property in Property::ALL {
            assert_eq!(Property::from_name(property.name()), Some(property));
        }

        let names: Vec<&str> = Property::ALL.iter().map(|p| p.name()).collect();

        assert_eq!(
            names,
            ["body", "definition", "foreign_keys", "referenced_by"]
        );
    }

    #[test]
    fn fr_priv_007_a_marking_that_records_nothing_seals_to_no_marking_at_all() {
        // FR-PRIV-007 with FR-PRIV-016: a complete object carries no
        // `restricted` key, and there is no empty array for it to carry.
        assert_eq!(Marking::default().sealed(), None);
    }

    #[test]
    fn fr_priv_016_the_marking_is_produced_in_byte_wise_ascending_order() {
        // FR-PRIV-016 with NFR-DET-002: the order is produced, not inherited
        // from the order the properties were observed in. The two are recorded
        // here in the order the fold reaches them — the referencing end after
        // the referenced one — and come back the other way round.
        let mut marking = Marking::default();

        marking.record(Property::ReferencedBy);
        marking.record(Property::ForeignKeys);

        let sealed = marking.sealed().expect("two properties are not empty");

        assert_eq!(
            sealed.properties(),
            [
                Cow::Borrowed("foreign_keys"),
                Cow::Borrowed("referenced_by")
            ]
        );
    }

    #[test]
    fn fr_priv_016_one_property_observed_many_times_is_named_once() {
        // Seventeen key columns produce one shortfall per table, not one per
        // column, and `restricted` names each property once.
        let mut marking = Marking::default();

        for _ in 0..17 {
            marking.record(Property::ForeignKeys);
        }

        let sealed = marking.sealed().expect("one property is not empty");

        assert_eq!(sealed.properties(), [Cow::Borrowed("foreign_keys")]);
    }

    #[test]
    fn fr_priv_003_a_named_object_that_is_marked_yields_the_verdict_that_owes_77() {
        // FR-PRIV-003 and FR-PRIV-004: the caller receives a verdict instead
        // of half an object, and the verdict's code is 77. The code is read
        // from the error rather than emitted here.
        let refused = of_view(&view(
            "",
            Restricted::new(vec![Cow::Borrowed("definition")]),
        ))
        .expect_err("a marked view is refused");

        assert_eq!(refused.exit_code(), 77);

        let Error::PropertyNotReadable {
            object, property, ..
        } = &refused
        else {
            panic!("{refused:?}");
        };

        assert_eq!(object, "v_consignment_manifest");
        assert_eq!(*property, "definition");
    }

    #[test]
    fn fr_priv_003_the_verdict_names_the_kind_each_object_was_sought_as() {
        // FR-ERR-034's 77 row obliges the kind, and the three entry points are
        // what fix it: a caller cannot name a routine and be told about a view.
        let refusals = [
            of_table(&table(Restricted::new(vec![Cow::Borrowed("foreign_keys")]))),
            of_view(&view(
                "",
                Restricted::new(vec![Cow::Borrowed("definition")]),
            )),
            of_routine(&routine("", Restricted::new(vec![Cow::Borrowed("body")]))),
        ];
        let kinds: Vec<String> = refusals
            .iter()
            .map(|refusal| {
                let failure = refusal.as_ref().expect_err("each object is marked");
                let Error::PropertyNotReadable { kind, .. } = failure else {
                    panic!("{failure:?}");
                };

                kind.to_string()
            })
            .collect();

        assert_eq!(kinds, ["table", "view", "routine"]);
    }

    #[test]
    fn fr_priv_004_a_complete_named_object_produces_no_verdict() {
        // FR-PRIV-007 seen from the caller: nothing was attempted and
        // refused, so there is nothing to refuse the caller either.
        assert!(of_table(&table(None)).is_ok());
        assert!(of_view(&view("select 1 AS `one`", None)).is_ok());
        assert!(of_routine(&routine("BEGIN END", None)).is_ok());
    }

    #[test]
    fn fr_priv_013_an_object_that_lost_two_properties_names_the_byte_wise_least() {
        // FR-PRIV-013 states one property, and NFR-DET-002 decides which of
        // two it is, so two runs produce the same message.
        let mut marking = Marking::default();

        marking.record(Property::ReferencedBy);
        marking.record(Property::ForeignKeys);

        let refused = of_table(&table(marking.sealed())).expect_err("a marked table is refused");
        let Error::PropertyNotReadable { property, .. } = &refused else {
            panic!("{refused:?}");
        };

        assert_eq!(*property, "foreign_keys");
    }

    #[test]
    fn fr_priv_014_no_line_of_the_message_carries_a_credential() {
        // FR-PRIV-014 through FR-ERR-013. The error value has no field a
        // credential could occupy, and the four rendered lines are read here
        // rather than argued about: the fixture's own passwords are searched
        // for by value.
        let refused = of_routine(&routine("", Restricted::new(vec![Cow::Borrowed("body")])))
            .expect_err("a marked routine is refused");
        let rendered = crate::diagnostics::rendered(&refused);

        for credential in ["tpl-reader-pw", "tpl-root", "password"] {
            assert!(!rendered.contains(credential), "{rendered}");
        }

        assert!(rendered.contains("the body of routine 'sp_book_consignment'"));
        assert!(rendered.contains("77 (EX_NOPERM)"));
    }
}
