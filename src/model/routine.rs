//! A stored procedure or a stored function, and its parameters (`FR-CAT-008`,
//! `FR-CAT-016` … `FR-CAT-018`, `FR-CAT-048`, `FR-CAT-049`).
//!
//! Sixteen properties from a catalogue field list of 31. Two fields are row
//! identity; the specific name restates the routine name; the external name,
//! the external language and the SQL path were never observed populated and are
//! not carried under `BR-CAT-005`; and the creation and alteration timestamps
//! are volatile under `FR-CAT-024`, for the reason that decided it — they are
//! wall-clock times that differ between two containers of the **same** series,
//! so a document carrying them could never be byte-identical across four
//! servers.
//!
//! *A function's return value is a row of the parameter catalogue and is not a
//! parameter.* `FR-CAT-049` puts it at ordinal position `0`, with both the mode
//! and the name SQL `NULL`, while declared parameters start at `1` and a
//! procedure has no row at `0`. The return type this module carries is
//! [`Routine::return_type`], read from the routine row per `FR-CAT-048`, and
//! [`RoutineParameter`] is the declared parameters alone — which is why its
//! name and its mode are not optional.
//!
//! *The two absent-value shapes of a procedure sit side by side and are
//! different.* Its data-type field is the **empty string** while the DTD
//! identifier beside it is SQL `NULL`, so a reader that tests one shape finds
//! the other populated. The model emits the whole return type as `null`, per
//! `FR-CTX-005`.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::column_type::ColumnType;
use super::restricted::Restricted;

/// Whether a routine is a procedure or a function (`FR-CAT-016`).
///
/// `FR-CAT-008` covers the two together and `FR-CAT-016` requires each object
/// to state which it is. The two spellings below are the catalogue's; the lower
/// case forms `procedure:` and `function:` that `FR-SCH-008` accepts on the
/// command line are a matter for the parser and not for the model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum RoutineKind {
    /// A stored procedure. It has no return type.
    Procedure,

    /// A stored function.
    Function,
}

impl RoutineKind {
    /// Reads the kind from the routine-type field.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            "PROCEDURE" => Some(Self::Procedure),
            "FUNCTION" => Some(Self::Function),
            _ => None,
        }
    }

    /// The spelling the catalogue writes, which is what the document carries.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Procedure => "PROCEDURE",
            Self::Function => "FUNCTION",
        }
    }
}

/// One declared parameter of a routine (`FR-CAT-049`).
///
/// The ordinal position is not a field: it is the declaration order
/// `FR-CAT-018` requires, and the order of
/// [`Routine::parameters`] is that order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RoutineParameter<'a> {
    /// The parameter's name. It is SQL `NULL` only on the return row, which is
    /// not a parameter, so it is never absent here.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// The mode, carried verbatim. `IN`, `OUT` and `INOUT` were observed; a
    /// function's parameters are declared with no mode keyword, because
    /// MariaDB accepts none, and are reported as `IN`.
    pub mode: Cow<'a, str>,

    /// The parameter's type, decomposed exactly as `FR-CTX-015` decomposes a
    /// column's — the catalogue reports it through the same fields.
    pub parameter_type: ColumnType<'a>,
}

/// One stored procedure or stored function (`FR-CAT-048`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Routine<'a> {
    /// The routine's name.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// Whether it is a procedure or a function (`FR-CAT-016`).
    pub kind: RoutineKind,

    /// The return type of a function, or [`None`] in full for a procedure
    /// (`FR-CAT-048`).
    pub return_type: Option<ColumnType<'a>>,

    /// The declared parameters, in declaration order (`FR-CAT-018`), which is
    /// one of the six exceptions of `NFR-DET-002`.
    #[serde(borrow)]
    pub parameters: Vec<RoutineParameter<'a>>,

    /// The body **as written**, with newlines and identifier case preserved
    /// (`FR-CAT-017`) — and **not** rewritten as a view definition is.
    pub body: Cow<'a, str>,

    /// The routine body kind, carried verbatim. It read `SQL` on all seven
    /// routines of the fixture.
    pub body_kind: Cow<'a, str>,

    /// The parameter style, carried verbatim. It read `SQL` on all seven.
    pub parameter_style: Cow<'a, str>,

    /// Whether the routine is deterministic, read from a field that returns
    /// `YES` or `NO`; both were observed.
    pub is_deterministic: bool,

    /// The SQL data access, carried verbatim. `READS SQL DATA`, `NO SQL` and
    /// `MODIFIES SQL DATA` were the three values returned.
    pub sql_data_access: Cow<'a, str>,

    /// The security type, which read `DEFINER` on all seven.
    pub security_type: Cow<'a, str>,

    /// The session SQL mode in force when the routine was created.
    pub sql_mode: Cow<'a, str>,

    /// The comment, which is the empty string where none was given.
    pub comment: Cow<'a, str>,

    /// The definer, which read one user across the fixture.
    pub definer: Cow<'a, str>,

    /// The session character set, passed through verbatim per `FR-SRV-039`.
    pub character_set_client: Cow<'a, str>,

    /// The session collation, passed through verbatim per `FR-SRV-039`. It
    /// differs between series, and is one of the two rows of the divergence
    /// register of `FR-SRV-036`.
    pub collation_connection: Cow<'a, str>,

    /// The schema's collation, passed through verbatim per `FR-SRV-039`. It
    /// follows the schema and does **not** differ between series.
    pub database_collation: Cow<'a, str>,

    /// The properties of this routine that could not be read, or [`None`]
    /// where it is complete (`FR-PRIV-005` … `FR-PRIV-007`).
    ///
    /// An unreadable body is the commonest case, and it is the example
    /// `FR-PRIV-016` gives. It needs no cross-check: the field comes back SQL
    /// `NULL` and is self-announcing, per `FR-PRIV-015`.
    ///
    /// The key is absent rather than `null` where the routine is complete, for
    /// the reason [`View::restricted`](super::view::View::restricted) gives.
    #[serde(borrow, default, skip_serializing_if = "Option::is_none")]
    pub restricted: Option<Restricted<'a>>,
}

#[cfg(test)]
mod tests {
    use super::{Routine, RoutineKind, RoutineParameter};
    use crate::model::column_type::{CatalogueType, ColumnType};
    use crate::model::restricted::Restricted;
    use std::borrow::Cow;

    fn decimal() -> ColumnType<'static> {
        ColumnType::decompose(&CatalogueType {
            column_type: "decimal(12,2)",
            data_type: "decimal",
            numeric_precision: Some(12),
            numeric_scale: Some(2),
            ..CatalogueType::default()
        })
    }

    fn routine(kind: RoutineKind, return_type: Option<ColumnType<'static>>) -> Routine<'static> {
        Routine {
            name: Cow::Borrowed("sp_book_consignment"),
            kind,
            return_type,
            parameters: vec![RoutineParameter {
                name: Cow::Borrowed("p_declared_value"),
                mode: Cow::Borrowed("IN"),
                parameter_type: decimal(),
            }],
            body: Cow::Borrowed("BEGIN\n  SELECT 1;\nEND"),
            body_kind: Cow::Borrowed("SQL"),
            parameter_style: Cow::Borrowed("SQL"),
            is_deterministic: false,
            sql_data_access: Cow::Borrowed("MODIFIES SQL DATA"),
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

    #[test]
    fn fr_cat_008_the_two_kinds_round_trip_and_nothing_else_is_a_kind() {
        // FR-CAT-008 covers procedures and functions together; FR-CAT-016
        // requires each object to state which it is.
        for kind in [RoutineKind::Procedure, RoutineKind::Function] {
            assert_eq!(RoutineKind::from_catalogue(kind.name()), Some(kind));
        }

        assert_eq!(RoutineKind::from_catalogue("TRIGGER"), None);
        assert_eq!(RoutineKind::from_catalogue("procedure"), None);
    }

    #[test]
    fn fr_cat_048_a_procedure_carries_no_return_type_and_a_function_carries_one() {
        // FR-CAT-048: the model emits the whole return type as `null` for a
        // procedure, per FR-CTX-005, rather than carrying the two different
        // absent values the catalogue reports side by side.
        assert_eq!(routine(RoutineKind::Procedure, None).return_type, None);
        assert_eq!(
            routine(RoutineKind::Function, Some(decimal())).return_type,
            Some(decimal())
        );
    }

    #[test]
    fn fr_cat_049_a_parameter_carries_a_name_and_a_mode_because_the_return_row_is_not_one() {
        // FR-CAT-049: the return row is at ordinal position 0 with both fields
        // SQL NULL, and it is not presented as a parameter. Every value that
        // reaches this type is therefore a declared parameter.
        let declared = routine(RoutineKind::Function, Some(decimal()));
        let parameter = &declared.parameters[0];

        assert_eq!(parameter.name, "p_declared_value");
        assert_eq!(parameter.mode, "IN");
        assert_eq!(parameter.parameter_type.data_type(), Some("decimal"));
    }

    #[test]
    fn fr_priv_016_an_unreadable_body_is_the_marking_the_requirement_gives_as_its_example() {
        // FR-PRIV-016: `{"name":"sp_book_consignment","restricted":["body"]}`.
        let incomplete = Routine {
            restricted: Restricted::new(vec![Cow::Borrowed("body")]),
            ..routine(RoutineKind::Procedure, None)
        };

        let marking = incomplete
            .restricted
            .as_ref()
            .expect("the routine was built incomplete");

        assert_eq!(marking.properties(), [Cow::Borrowed("body")]);
    }

    #[test]
    fn fr_cat_048_the_body_is_carried_as_written() {
        // FR-CAT-048: newlines and identifier case preserved, unlike a view
        // definition, which the server rewrites.
        let procedure = routine(RoutineKind::Procedure, None);

        assert!(procedure.body.contains('\n'));
        assert!(procedure.body.starts_with("BEGIN"));
    }
}
