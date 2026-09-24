//! The model: a database's structure as `tpl` carries it.
//!
//! This is the junction the tool turns on. Whatever produced a model — a live
//! read, the cache, or a document supplied by the caller — it holds the same
//! shape here, which is what lets one template be written against one answer.
//!
//! At this commit the module owns the **object graph** and the **document that
//! carries it**: what the model holds for each covered object kind, what it is
//! unable to hold, and how the same material is written out and read back
//! without changing shape.
//!
//! | Submodule | What it owns |
//! |---|---|
//! | [`database`] | The root of `FR-CTX-035` and `FR-CTX-036` — three metadata fields, the server, and the three collections |
//! | [`server`] | The `server` object of `FR-CTX-031` … `FR-CTX-034`, and the series derivation of `FR-SRV-040` |
//! | [`table`] | The two covered table types of `FR-CAT-001`, everything a table carries per `FR-CAT-009` … `FR-CAT-015`, and the key invariant of `FR-CAT-044` |
//! | [`mod@column`] | A column per `FR-SCH-009` and `FR-CTX-019` … `FR-CTX-021`, with the static attributes of `FR-CAT-041` and the generated pair of `FR-CAT-051` |
//! | [`column_default`] | The discriminated default of `FR-CTX-011` … `FR-CTX-013`, and the eight-row shape classifier of `FR-CTX-037` |
//! | [`column_type`] | The raw type of `FR-CTX-014` and its eight decomposed parts, read as `FR-CTX-015` … `FR-CTX-018` and `FR-CTX-038` … `FR-CTX-041` fix them |
//! | [`index`] | An index folded to one object with an ordered column list, per `FR-CAT-010` and `FR-CAT-042` |
//! | [`foreign_key`] | Both directions of a foreign key per `FR-CAT-045`, with the four reachable rules and the positional pairing |
//! | [`check_constraint`] | A `CHECK` constraint per `FR-CAT-046`, and the two levels it is declared at |
//! | [`trigger`] | A trigger per `FR-CAT-050`, and the limits two of its fields carry |
//! | [`view`] | A view per `FR-CAT-047`, which carries nine fields and no comment |
//! | [`routine`] | A routine and its parameters per `FR-CAT-048` and `FR-CAT-049` |
//! | [`restricted`] | The `restricted` marking of `FR-PRIV-016`, which is never empty |
//! | `document` | The document of `FR-CTX-001` … `FR-CTX-010` in both directions: the orderings of `NFR-DET-002`, the one-hop embedding of `ADR-009`, and the read-back of `FR-CTX-033` |
//!
//! # What the model refuses, and why the refusal is structural
//!
//! `BR-CAT-005` states the rule the field lists were written by: the model
//! carries **every field the catalogue provides for an object** except a field
//! excluded on one of four grounds — row identity, volatility (`FR-CAT-024`),
//! ambiguous meaning (`FR-CAT-029`, whose list is empty) and restatement
//! (`FR-CTX-021`).
//!
//! An excluded field has **no field in the type**. It is not filtered on the
//! way out, and there is no place for it to be written into by a later reader,
//! a later emitter, or a caller assembling a value by hand. That is the
//! difference between a model that refuses and a model that merely omits: the
//! sixteen volatile fields of `FR-CAT-024` cannot enter, so `FR-CAT-026`'s
//! *every consumer without exception* holds by construction rather than by six
//! consumers each remembering.
//!
//! Three further refusals are structural in the same way, each recorded beside
//! the type that makes it: a rule the server discards silently has no variant
//! ([`foreign_key::ReferentialAction`]); a table type the model does not cover
//! has none either ([`table::TableType`]); and a key cannot name a column its
//! table does not carry ([`table::Table::assemble`]).
//!
//! # How the graph holds its strings
//!
//! Every string in the graph is a [`Cow<'a, str>`](Cow), under one lifetime
//! parameter threaded through every type. A model read from a live catalogue
//! borrows the row buffers; a model read from a document on disk borrows the
//! bytes where the encoding allows it and owns the value where it does not; and
//! `Cow<'static, str>` throughout is a model that borrows nothing and outlives
//! any source. One shape serves all three, which is what `FR-CAT-001`'s promise
//! — *the model is the same whatever the source* — requires of the types that
//! carry it.
//!
//! *Rejected: owned [`String`] everywhere.* It is the simplest shape and it
//! costs a copy of every catalogue string on the live-read path, which is the
//! dominant one. It also does not remove the lifetime: [`column_type::ColumnType`]
//! and [`column_default::ColumnDefault`] borrow, so a column carrying either
//! carries their lifetime, and an owned graph would need owned twins of both —
//! the second parallel type this shape exists to avoid.
//!
//! *Rejected: `&'a str` everywhere.* Cheaper still, and it cannot represent two
//! values the model already produces: a string literal whose doubled apostrophe
//! was collapsed, and a document string that needed unescaping. Neither is a
//! slice of its source.
//!
//! *Rejected: a type parameter for the string type, `Table<S: AsRef<str>>`.* It
//! defers the choice to every caller and infects every type with a parameter,
//! multiplies the monomorphisations, and leaves the render environment with two
//! concrete models to be written against instead of one.
//!
//! *Rejected: an arena the whole graph borrows from.* It would remove the
//! per-string allocation on the owned path, at the price of a dependency the
//! budget would have to justify and of a construction path
//! [`serde::Deserialize`] cannot express — the arena would have to be threaded
//! through as a seed, which is exactly the plain derive this choice keeps
//! available.
//!
//! # Where the types are built
//!
//! A type with an invariant to enforce has private fields and a constructor
//! that establishes it — [`table::Table`], [`server::Server`] and
//! [`restricted::Restricted`]. Every other type is plain data with public
//! fields and `#[non_exhaustive]`, so the model can be read anywhere and built
//! only inside this crate. It is the division [`column_type`] already draws
//! between its input and its product.
//!
//! **Only one of the additions can fail**, and it fails for one condition:
//! [`table::Table::assemble`] refuses a key that names a column its table does
//! not carry, per `FR-CAT-044`. It reports [`table::UnknownKeyColumn`] rather
//! than a variant of [`crate::Error`], because the same violation is an
//! internal invariant on one path and caller data on the other, and the error
//! type would have to choose. `FR-CTX-037` and `FR-CTX-018` keep the two
//! decompositions total, so nothing else here reports failure.

/// Defines an enumeration whose recorded values a requirement fixes and whose
/// unrecorded ones are carried verbatim (`FR-CAT-055`).
///
/// Five catalogue fields take a set of values that a requirement closed from an
/// observation of all four series of `FR-SRV-015`, and the catalogue closes
/// none of them: every field behind the five is a plain `varchar` and never an
/// `ENUM`, so a server outside the window may return a sixth value at any time.
/// `FR-CAT-055` fixes what happens then — the catalogue's own string is carried
/// unchanged, the object is not dropped, the read is not refused, and the exit
/// code does not move — and the shape that obliges all four at once is an
/// enumeration with a variant for each recorded value and one that carries the
/// string.
///
/// **The macro exists because the five would otherwise be five copies of one
/// decision.** The reading, the spelling, the serialisation and the read-back
/// are identical for all five and differ only in the table of spellings, so
/// they are written once here and each enumeration supplies its own table —
/// the same reason `mariadb::catalogue::statements` writes its column
/// lists through a macro. A second copy of a rule is a second thing that can
/// be wrong.
///
/// The document is unchanged for every value any server has been observed to
/// return: a recorded variant serialises to the spelling its requirement fixes,
/// which is the byte sequence the catalogue itself returned, and only a value
/// outside the set reaches the carried string. The read-back accepts both, so a
/// document written from a server newer than the window survives a round trip
/// through the cache with the value the server stated.
macro_rules! catalogued {
    (
        $(#[$enumeration:meta])*
        $name:ident from $field:literal {
            $( $(#[$recorded:meta])* $variant:ident = $spelling:literal, )+
        }
    ) => {
        $(#[$enumeration])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        pub enum $name<'a> {
            $( $(#[$recorded])* $variant, )+

            /// A value outside the recorded set, carried exactly as the
            /// catalogue returned it (`FR-CAT-055`).
            ///
            /// No read of a series of `FR-SRV-015` has produced one. The
            /// foreseeable server that would is one newer than the window,
            /// which `FR-SRV-031` reads and marks rather than refuses, and
            /// `FR-CTX-034`'s `standing` is the field that says the read is
            /// unverified. Substituting a recorded value or dropping the
            /// object that carries the field are the two outcomes `FR-CAT-055`
            /// rejects by name, both of them being wrong documents at exit `0`.
            Unrecorded(Cow<'a, str>),
        }

        impl<'a> $name<'a> {
            #[doc = concat!(
                "Reads the value from the catalogue's `", $field, "` field, \
                 borrowed from the row."
            )]
            ///
            /// It is total, per `FR-CAT-055`: a value outside the recorded set
            /// is carried rather than refused.
            #[must_use]
            pub fn from_catalogue(field: &'a str) -> Self {
                match field {
                    $( $spelling => Self::$variant, )+
                    carried => Self::Unrecorded(Cow::Borrowed(carried)),
                }
            }

            /// The same reading over a value the caller owns, which is the
            /// shape a document read back from disk arrives in.
            #[must_use]
            pub fn from_catalogue_owned(field: String) -> Self {
                match field.as_str() {
                    $( $spelling => Self::$variant, )+
                    _ => Self::Unrecorded(Cow::Owned(field)),
                }
            }

            /// The spelling the model carries: the one the requirement fixes
            /// for a recorded value, and the catalogue's own string otherwise.
            #[must_use]
            pub fn name(&self) -> &str {
                match self {
                    $( Self::$variant => $spelling, )+
                    Self::Unrecorded(carried) => carried,
                }
            }

            /// The recorded spelling, or [`None`] where the value is outside
            /// the set.
            ///
            /// It is [`name`](Self::name) narrowed to the values a requirement
            /// fixes, for the callers that may use a value only where the
            /// corpus has written down what it means — composing a path, or
            /// binding it to a statement that narrows on it. `FR-CAT-055`
            /// names the one such place a caller can reach: the qualified
            /// routine name of `FR-SCH-008`, which an unrecorded kind is not
            /// reachable by.
            #[must_use]
            pub const fn recorded(&self) -> Option<&'static str> {
                match self {
                    $( Self::$variant => Some($spelling), )+
                    Self::Unrecorded(_) => None,
                }
            }
        }

        impl serde::Serialize for $name<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.name())
            }
        }

        impl<'de, 'a> serde::Deserialize<'de> for $name<'a> {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                // The value is taken as an owned `String` rather than borrowed
                // from the input. A recorded value becomes a unit variant and
                // the allocation is dropped at once, which is every value any
                // server has returned; only the unrecorded one keeps it, and
                // `FR-CAT-055` records that nothing observed reaches it.
                <String as serde::Deserialize<'de>>::deserialize(deserializer)
                    .map(Self::from_catalogue_owned)
            }
        }

        impl $crate::model::ToStatic for $name<'_> {
            type Static = $name<'static>;

            fn to_static(&self) -> Self::Static {
                match self {
                    $( Self::$variant => $name::$variant, )+
                    Self::Unrecorded(carried) => {
                        $name::Unrecorded($crate::model::ToStatic::to_static(carried))
                    }
                }
            }
        }
    };
}

pub mod check_constraint;
pub mod column;
pub mod column_default;
pub mod column_type;
pub mod database;

// Nothing calls either direction yet: `tpl schema dump` emits the document and
// `tpl render --context` consumes it, and both commands are a later sprint.
// One fact explains every item of the module, so it is stated once here rather
// than once per item.
#[allow(
    dead_code,
    reason = "the two commands that carry this document — `tpl schema dump` and `tpl render \
              --context` — are a later sprint, and the block that owns the model owns the \
              document it is written as"
)]
pub(crate) mod document;

pub mod foreign_key;
pub mod index;
pub mod restricted;
pub mod routine;
pub mod server;
pub mod table;
pub mod trigger;
pub mod view;

use std::borrow::Cow;

/// A value copied into one that borrows nothing (`Cow<'static, str>` throughout).
///
/// The render context needs it. `minijinja` holds an object behind an
/// `Arc<dyn Object + 'static>`, so a document it reads **on demand** — rather
/// than converting it whole before the template runs — must own everything the
/// template may reach. Every string is copied once, and nothing else is built:
/// no map, no key and no `minijinja` value, which is what a whole conversion
/// costs.
///
/// It is a trait rather than a method on each type so that the three
/// containers the graph uses — [`Cow`] of a slice, [`Vec`] and [`Option`] —
/// are written once, and so that the two instantiations of the document's
/// table shape, which differ only by what a reference resolves to, are one
/// generic implementation.
///
/// *Rejected: a `Clone` into a wider lifetime.* `Clone` keeps the lifetime of a
/// borrowed [`Cow`], which is precisely the borrow that must end.
pub(crate) trait ToStatic {
    /// The same shape, borrowing nothing.
    type Static: 'static;

    /// Copies every borrowed string, and moves nothing out of `self`.
    fn to_static(&self) -> Self::Static;
}

impl ToStatic for Cow<'_, str> {
    type Static = Cow<'static, str>;

    fn to_static(&self) -> Self::Static {
        Cow::Owned(self.as_ref().to_owned())
    }
}

impl<T: ToStatic> ToStatic for Option<T> {
    type Static = Option<T::Static>;

    fn to_static(&self) -> Self::Static {
        self.as_ref().map(ToStatic::to_static)
    }
}

impl<T: ToStatic> ToStatic for Vec<T> {
    type Static = Vec<T::Static>;

    fn to_static(&self) -> Self::Static {
        self.iter().map(ToStatic::to_static).collect()
    }
}

impl<T> ToStatic for Cow<'_, [T]>
where
    T: ToStatic + Clone,
    T::Static: Clone,
{
    type Static = Cow<'static, [T::Static]>;

    fn to_static(&self) -> Self::Static {
        Cow::Owned(self.iter().map(ToStatic::to_static).collect())
    }
}

/// The two characters the catalogue writes where a quoted value carries one
/// apostrophe.
const DOUBLED_APOSTROPHE: &str = "''";

/// The one character they stand for.
const APOSTROPHE: &str = "'";

/// Collapses each doubled apostrophe of a quoted value to one.
///
/// One convention, written once, because the catalogue applies one:
/// `FR-CTX-037` fixes it for the value of a `literal` default and states that
/// it is the same convention `FR-CTX-039` fixes for a member of an `ENUM` or a
/// `SET`. Both were observed to double the apostrophe and **never** to
/// backslash-escape it — there is no `5C` byte anywhere in the observed values
/// — so there is no escape byte to strip and nothing else to undo.
///
/// The argument is the text **between** the delimiting quotes; the caller has
/// already found them, because finding them is the part the two callers do
/// differently.
///
/// The value is borrowed where it carries no doubled apostrophe, which is the
/// common case: a copy is made only where a byte has to be dropped.
pub(crate) fn collapse_doubled_apostrophes(quoted: &str) -> Cow<'_, str> {
    if quoted.contains(DOUBLED_APOSTROPHE) {
        Cow::Owned(quoted.replace(DOUBLED_APOSTROPHE, APOSTROPHE))
    } else {
        Cow::Borrowed(quoted)
    }
}

#[cfg(test)]
mod tests {
    use super::collapse_doubled_apostrophes;
    use std::borrow::Cow;

    #[test]
    fn a_value_carrying_no_doubled_apostrophe_is_borrowed_rather_than_copied() {
        assert_eq!(collapse_doubled_apostrophes("EUR"), Cow::Borrowed("EUR"));
        assert!(matches!(
            collapse_doubled_apostrophes("EUR"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn fr_ctx_037_each_doubled_apostrophe_collapses_to_one_and_every_other_byte_passes_through() {
        // FR-CTX-037 and FR-CTX-039 share this convention. The double quote of
        // the fixture's `8'6"` is left alone, and the em dash survives whole.
        assert_eq!(collapse_doubled_apostrophes(r#"8''6""#), "8'6\"");
        assert_eq!(
            collapse_doubled_apostrophes("Lloyd''s Register"),
            "Lloyd's Register"
        );
        assert_eq!(
            collapse_doubled_apostrophes("Method 1 — weighbridge"),
            "Method 1 — weighbridge"
        );
        assert_eq!(collapse_doubled_apostrophes("''''"), "''");
    }
}

/// The refusals of `FR-CAT-024`, `FR-CAT-026` and `FR-CTX-021`, checked over
/// every type of the graph.
///
/// A field that is absent cannot be asserted about at run time by reading it —
/// that is the whole point of the refusal being structural. What **can** be
/// read is the derived [`Debug`] representation, which names every field of
/// every value, so a sweep of it over one instance of each type is a direct
/// test of the property the requirements state: that no type carries the field,
/// anywhere, under any name.
#[cfg(test)]
mod refusals {
    use std::borrow::Cow;

    use super::check_constraint::{CheckConstraint, ConstraintLevel};
    use super::column::{Column, Generated, GeneratedStorage};
    use super::column_type::{CatalogueType, ColumnType};
    use super::database::Database;
    use super::foreign_key::{ForeignKey, ForeignKeyColumn, IncomingForeignKey, ReferentialAction};
    use super::index::{Index, IndexColumn, PRIMARY_KEY_NAME, SortDirection};
    use super::restricted::Restricted;
    use super::routine::{Routine, RoutineKind, RoutineParameter};
    use super::server::{Server, Standing};
    use super::table::{Table, TableParts, TableType};
    use super::trigger::{Trigger, TriggerEvent, TriggerTiming};
    use super::view::View;

    /// The sixteen fields of `FR-CAT-024`, as a field of the model would have
    /// to be named. `CREATED` is on the list twice — once for a routine and
    /// once for a trigger — so fifteen names cover the sixteen rows.
    const VOLATILE: [&str; 15] = [
        "table_rows",
        "avg_row_length",
        "data_length",
        "max_data_length",
        "index_length",
        "data_free",
        "auto_increment",
        "create_time",
        "update_time",
        "check_time",
        "checksum",
        "version",
        "cardinality",
        "created",
        "last_altered",
    ];

    /// The two fields `FR-CTX-021` names in as many words.
    const MATERIALISED: [&str; 2] = ["is_primary_key", "is_unique"];

    fn column_type() -> ColumnType<'static> {
        ColumnType::decompose(&CatalogueType {
            column_type: "varchar(64)",
            data_type: "varchar",
            character_maximum_length: Some(64),
            charset: Some("utf8mb4"),
            collation: Some("utf8mb4_unicode_520_ci"),
            ..CatalogueType::default()
        })
    }

    fn column() -> Column<'static> {
        Column {
            name: Cow::Borrowed("reference"),
            table_name: Cow::Borrowed("consignment"),
            position: 2,
            column_type: column_type(),
            nullable: false,
            default: None,
            comment: Cow::Borrowed("the carrier's own reference"),
            auto_increment: false,
            invisible: false,
            generated: Some(Generated {
                expression: Cow::Borrowed("upper(`reference`)"),
                storage: GeneratedStorage::Stored,
            }),
            on_update: Some(Cow::Borrowed("on update current_timestamp()")),
        }
    }

    fn index_column() -> IndexColumn<'static> {
        IndexColumn {
            name: Cow::Borrowed("reference"),
            direction: Some(SortDirection::Ascending),
            prefix_length: Some(32),
        }
    }

    fn index() -> Index<'static> {
        Index {
            name: Cow::Borrowed(PRIMARY_KEY_NAME),
            unique: true,
            columns: vec![index_column()],
            index_type: Cow::Borrowed("BTREE"),
            comment: Cow::Borrowed("the natural key"),
            ignored: false,
        }
    }

    fn foreign_key_column() -> ForeignKeyColumn<'static> {
        ForeignKeyColumn {
            column: Cow::Borrowed("reference"),
            referenced_column: Cow::Borrowed("reference"),
        }
    }

    fn foreign_key() -> ForeignKey<'static> {
        ForeignKey {
            name: Cow::Borrowed("fk_leg_consignment"),
            columns: vec![foreign_key_column()],
            referenced_table: Some(Cow::Borrowed("consignment")),
            referenced_key: Some(Cow::Borrowed(PRIMARY_KEY_NAME)),
            match_option: Cow::Borrowed("NONE"),
            on_update: ReferentialAction::Cascade,
            on_delete: ReferentialAction::SetNull,
        }
    }

    fn incoming_foreign_key() -> IncomingForeignKey<'static> {
        IncomingForeignKey {
            table: Cow::Borrowed("consignment_leg"),
            key: foreign_key(),
        }
    }

    fn check_constraint() -> CheckConstraint<'static> {
        CheckConstraint {
            name: Cow::Borrowed("ck_consignment_reference"),
            level: ConstraintLevel::Table,
            clause: Cow::Borrowed("char_length(`reference`) > 0"),
        }
    }

    fn trigger() -> Trigger<'static> {
        Trigger {
            name: Cow::Borrowed("trg_consignment_stamp"),
            event: TriggerEvent::Insert,
            timing: TriggerTiming::Before,
            action_order: 1,
            statement: Some(Cow::Borrowed(
                "BEGIN SET NEW.reference = upper(NEW.reference); END",
            )),
            orientation: Cow::Borrowed("ROW"),
            old_row_alias: Cow::Borrowed("OLD"),
            new_row_alias: Cow::Borrowed("NEW"),
            sql_mode: Cow::Borrowed("STRICT_TRANS_TABLES"),
            definer: Some(Cow::Borrowed("root@localhost")),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
            database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
        }
    }

    fn view() -> View<'static> {
        View {
            name: Cow::Borrowed("v_consignment_manifest"),
            definition: Cow::Borrowed("select `freight`.`consignment`.`reference` AS `reference`"),
            check_option: Cow::Borrowed("CASCADED"),
            is_updatable: true,
            definer: Cow::Borrowed("root@localhost"),
            security_type: Cow::Borrowed("INVOKER"),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
            algorithm: Cow::Borrowed("UNDEFINED"),
            restricted: Restricted::new(vec![Cow::Borrowed("definition")]),
        }
    }

    fn routine_parameter() -> RoutineParameter<'static> {
        RoutineParameter {
            name: Cow::Borrowed("p_reference"),
            mode: Cow::Borrowed("INOUT"),
            parameter_type: column_type(),
        }
    }

    fn routine() -> Routine<'static> {
        Routine {
            name: Cow::Borrowed("fn_normalise_reference"),
            kind: RoutineKind::Function,
            return_type: Some(column_type()),
            parameters: vec![routine_parameter()],
            body: Some(Cow::Borrowed("BEGIN RETURN upper(p_reference); END")),
            body_kind: Cow::Borrowed("SQL"),
            parameter_style: Cow::Borrowed("SQL"),
            is_deterministic: true,
            sql_data_access: Cow::Borrowed("NO SQL"),
            security_type: Cow::Borrowed("DEFINER"),
            sql_mode: Cow::Borrowed("STRICT_TRANS_TABLES"),
            comment: Cow::Borrowed(""),
            definer: Cow::Borrowed("root@localhost"),
            character_set_client: Cow::Borrowed("utf8mb4"),
            collation_connection: Cow::Borrowed("utf8mb4_uca1400_ai_ci"),
            database_collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
            restricted: None,
        }
    }

    /// A table carrying its own row's fields and **no members**.
    ///
    /// The collections are deliberately empty. A table holding a column would
    /// print that column's field names inside its own [`Debug`] output, and the
    /// sweep would then report the table for a field of the column — which is
    /// exactly the confusion `FR-CAT-027` guards against for the one name the
    /// two kinds share.
    fn table() -> Table<'static> {
        Table::assemble(TableParts {
            engine: Some(Cow::Borrowed("InnoDB")),
            collation: Some(Cow::Borrowed("utf8mb4_unicode_520_ci")),
            comment: Cow::Borrowed("one consignment per row"),
            restricted: Restricted::new(vec![Cow::Borrowed("columns")]),
            ..TableParts::new(Cow::Borrowed("consignment"), TableType::SystemVersioned)
        })
        .expect("a table with no keys satisfies FR-CAT-044 vacuously")
    }

    /// The same table, carrying one member of every kind.
    ///
    /// It is what proves the members are reachable at all, so that the sweep
    /// over the shallow value is not a sweep over a type nothing uses.
    fn populated_table() -> Table<'static> {
        Table::assemble(TableParts {
            columns: vec![column()],
            indexes: vec![index()],
            foreign_keys: vec![foreign_key()],
            referenced_by: vec![incoming_foreign_key()],
            triggers: vec![trigger()],
            check_constraints: vec![check_constraint()],
            ..TableParts::new(Cow::Borrowed("consignment"), TableType::SystemVersioned)
        })
        .expect("every key of the fixture names the one column it carries")
    }

    fn database() -> Database<'static> {
        Database {
            name: Cow::Borrowed("freight"),
            charset: Cow::Borrowed("utf8mb4"),
            collation: Cow::Borrowed("utf8mb4_unicode_520_ci"),
            server: Server::probed(
                Cow::Borrowed("11.4.13-MariaDB-ubu2404"),
                Standing::Supported,
            )
            .expect("the observed form yields a series"),
            tables: vec![populated_table()],
            views: vec![view()],
            routines: vec![routine()],
        }
    }

    /// One fully populated instance of every type that carries a catalogue
    /// row, named so a failure says which type carries the field.
    ///
    /// Each is built shallow — the nested collections of a table are its own
    /// row's business, and a sweep over a nested value would report the
    /// containing type for a field of the contained one.
    fn catalogue_objects() -> Vec<(&'static str, String)> {
        vec![
            ("Table", format!("{:#?}", table())),
            ("Column", format!("{:#?}", column())),
            ("Index", format!("{:#?}", index())),
            ("IndexColumn", format!("{:#?}", index_column())),
            ("ForeignKey", format!("{:#?}", foreign_key())),
            ("ForeignKeyColumn", format!("{:#?}", foreign_key_column())),
            (
                "IncomingForeignKey",
                format!("{:#?}", incoming_foreign_key()),
            ),
            ("CheckConstraint", format!("{:#?}", check_constraint())),
            ("Trigger", format!("{:#?}", trigger())),
            ("View", format!("{:#?}", view())),
            ("Routine", format!("{:#?}", routine())),
            ("RoutineParameter", format!("{:#?}", routine_parameter())),
        ]
    }

    #[test]
    fn fr_cat_009_a_table_carries_one_collection_for_each_of_the_seven_facts_it_holds() {
        // FR-CAT-009 through FR-CAT-015, and FR-CAT-011 with FR-CAT-043: the
        // primary key is not an eighth collection but the index named PRIMARY
        // inside the index one.
        let table = populated_table();

        assert_eq!(table.columns().len(), 1);
        assert_eq!(table.indexes().len(), 1);
        assert_eq!(table.foreign_keys().len(), 1);
        assert_eq!(table.referenced_by().len(), 1);
        assert_eq!(table.triggers().len(), 1);
        assert_eq!(table.check_constraints().len(), 1);
        assert!(table.primary_key().is_some());
    }

    #[test]
    fn fr_cat_024_no_type_carries_any_of_the_sixteen_volatile_fields() {
        // FR-CAT-024 excludes them anywhere, under any name, and FR-CAT-026
        // applies the exclusion to every consumer without exception. There is
        // no field to filter out downstream, which is what makes FR-CAT-026
        // hold by construction.
        //
        // The one exception is named by a requirement rather than assumed:
        // FR-CAT-027 keeps the column **attribute** `auto_increment`, and
        // FR-CAT-024 excludes only the table-level counter of the same name.
        // The next test is what proves the two are not confused.
        for (object, debug) in catalogue_objects() {
            for field in VOLATILE {
                if object == "Column" && field == "auto_increment" {
                    continue;
                }

                assert!(
                    !debug.contains(&format!("{field}:")),
                    "{object} carries the volatile field '{field}', which FR-CAT-024 excludes"
                );
            }
        }
    }

    #[test]
    fn fr_cat_027_the_auto_increment_the_model_keeps_is_the_column_attribute_and_not_the_table_counter()
     {
        // FR-CAT-027 against FR-CAT-024: one word, two things. The attribute
        // is a field of a column and the counter is a field of nothing.
        let column = format!("{:#?}", column());
        let table = format!("{:#?}", table());

        assert!(column.contains("auto_increment:"));
        assert!(!table.contains("auto_increment: "));
    }

    #[test]
    fn fr_ctx_021_no_type_materialises_is_primary_key_or_is_unique() {
        // FR-CTX-021 names both in as many words, and BR-CTX-003 is the
        // reason: a column that said it was not part of the primary key while
        // its table said it was would be a document that contradicts itself,
        // and nothing downstream could repair it once both were written.
        // FR-CTX-022 answers both from the table instead.
        let mut sweep = catalogue_objects();
        sweep.push(("Database", format!("{:#?}", database())));

        for (object, debug) in sweep {
            for field in MATERIALISED {
                assert!(
                    !debug.contains(&format!("{field}:")),
                    "{object} materialises '{field}', which FR-CTX-021 forbids"
                );
            }
        }
    }

    #[test]
    fn fr_cat_024_the_version_the_server_carries_is_the_probes_string_and_not_the_excluded_field() {
        // The `VERSION` of FR-CAT-024 is a field of the **table** catalogue,
        // and no type of the graph carries it. The `version` of FR-CTX-031 is
        // the string the probe of FR-SRV-002 returns; BR-CTX-006 records that
        // `server` does not come from the catalogue at all, so the two names
        // meet nowhere.
        let server = Server::probed(
            Cow::Borrowed("11.4.13-MariaDB-ubu2404"),
            Standing::Supported,
        )
        .expect("the observed form yields a series");

        assert_eq!(server.version(), "11.4.13-MariaDB-ubu2404");
        assert!(!format!("{:#?}", table()).contains("version:"));
    }

    #[test]
    fn fr_priv_007_a_marking_is_absent_where_the_object_is_complete_and_names_a_property_where_it_is_not()
     {
        // FR-PRIV-007 and FR-PRIV-016, over the three kinds a marking can
        // reach: a table, a view and a routine.
        assert!(table().restricted().is_some());
        assert!(view().restricted.is_some());
        assert_eq!(routine().restricted, None);

        for marking in [table().restricted().cloned(), view().restricted.clone()]
            .into_iter()
            .flatten()
        {
            assert!(
                !marking.properties().is_empty(),
                "FR-PRIV-016: a marking that is present names at least one property"
            );
        }
    }
}
