//! A foreign key, in both the directions a table sees one (`FR-CAT-012`,
//! `FR-CAT-013`, `FR-CAT-045`).
//!
//! `FR-CAT-045` reads a foreign key from two catalogue tables, because neither
//! is sufficient: the referential-constraint table carries the rules and names
//! no column, and the key-column-usage table carries the columns and no rule.
//! What the join produces is one key with **two parallel column lists** —
//! the referencing columns and the referenced ones, paired by position.
//!
//! *The failure this module exists to prevent is the drift between those two
//! lists.* Two `Vec`s of names can be built to different lengths, can be sorted
//! independently, and can be filtered by one caller and not the other, and
//! every one of those makes a key that pairs the wrong columns without saying
//! so. There is therefore **one** list here, of [`ForeignKeyColumn`], each
//! member carrying both halves of one pair. The document's two arrays are
//! projections of it, and they cannot disagree because there is nothing for
//! them to disagree with.
//!
//! *The second thing this module refuses is a rule the server cannot return.*
//! `FR-CAT-033` observed that `SET DEFAULT` is accepted by all four series
//! without error or warning and then reported as `RESTRICT`, and that
//! `SHOW CREATE TABLE` omits the clause. It is unrepresentable rather than
//! unsupported, so [`ReferentialAction`] gives it no recorded variant: a
//! generator written against a fifth rule would carry a branch that never
//! executes.
//!
//! *What the module no longer refuses is a rule it has not seen.* The four
//! recorded spellings are what a server of `FR-SRV-015` returns, and the
//! catalogue declares both rule fields a plain `varchar(64)`, so nothing but
//! that observation closes the set. `FR-CAT-055` fixes the case it does not
//! close: a fifth spelling is carried as the catalogue's own string, under
//! [`ReferentialAction::Unrecorded`], and neither the object nor the exit code
//! moves on account of it.
//!
//! *The third thing is a key that names no table.* `FR-CAT-056` records that
//! the referential-constraint table declares its referenced-table name and its
//! unique-constraint name **nullable** on all four series, and both reach the
//! model as [`Option`] rather than as the empty string a `Cow` would have had
//! to carry. `FR-CTX-006` fixes what the document does with the first of them:
//! the key is carried whole and the place its embedded table occupies carries
//! `null`.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::ToStatic;

catalogued! {
    /// The referential action of `ON UPDATE` and `ON DELETE` (`FR-CAT-045`).
    ///
    /// The four spellings are contract surface, upper case, with a single space
    /// in `NO ACTION`. `SET DEFAULT` is not among them, per `FR-CAT-033`, and
    /// has no variant to be written into: it is unrepresentable rather than
    /// unsupported, because all four series accept it and then report
    /// `RESTRICT`, so no read can return it.
    ///
    /// *The set is closed by an observation and not by the catalogue.* Both
    /// rule fields are declared `varchar(64)` and `NOT NULL` on all four
    /// series, never an `ENUM`, so a value outside the four is carried verbatim
    /// under `FR-CAT-055` rather than refused.
    ReferentialAction from "UPDATE_RULE` and `DELETE_RULE" {
        /// The change is applied to the referencing rows.
        Cascade = "CASCADE",
        /// The change is refused, by the standard's spelling.
        NoAction = "NO ACTION",
        /// The change is refused, by MariaDB's own spelling.
        Restrict = "RESTRICT",
        /// The referencing columns are set to `NULL`.
        SetNull = "SET NULL",
    }
}

/// One pair of a foreign key's two column lists (`FR-CAT-045`).
///
/// The catalogue pairs them by position — the ordinal position of the
/// referencing column against the position in the unique constraint — and this
/// type is that pairing made one value, so the two halves cannot drift apart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ForeignKeyColumn<'a> {
    /// The referencing column, a column of the table the key is declared on.
    #[serde(borrow)]
    pub column: Cow<'a, str>,

    /// The referenced column, a column of [`ForeignKey::referenced_table`].
    ///
    /// The catalogue declares it nullable and returns SQL `NULL` on every row
    /// that is not a foreign key's — 37 of the fixture's 54 key-column rows.
    /// None of those rows reaches the model: `FR-CAT-045` restricts the read to
    /// the rows that name a referenced table, and `FR-CAT-056` records that
    /// those are exactly the rows on which this field is populated, all 17 of
    /// them. The field is therefore a string and not an [`Option`], and the
    /// reason is the read's population rather than a value.
    #[serde(borrow)]
    pub referenced_column: Cow<'a, str>,
}

/// One foreign key (`FR-CAT-045`).
///
/// The table the key is declared on is not a field: the key is carried on that
/// table, per `FR-CAT-012`, so naming it again would be the restatement
/// `FR-CTX-021` forbids. [`IncomingForeignKey`] is the one direction where the
/// name is not already carried, and it is the one that carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ForeignKey<'a> {
    /// The constraint name the DDL gave.
    pub name: Cow<'a, str>,

    /// The referencing and referenced columns, paired by position and ordered
    /// by the referencing column's ordinal position.
    pub columns: Vec<ForeignKeyColumn<'a>>,

    /// The referenced table — **the table only**, without a schema — or
    /// [`None`] where the catalogue named none (`FR-CAT-056`).
    ///
    /// The model carries the name. The one-level-deep embedding of
    /// `FR-CTX-006` is a rule of the document, applied where the document is
    /// shaped, and it resolves this name against the model it was built from.
    ///
    /// The field is declared nullable, `varchar(64)`, on all four series, and
    /// was populated on all fifteen rules of the fixture. A key that names no
    /// table is still carried, with its name, its columns and its rules, per
    /// `FR-CTX-006`; what it has no part in is the incoming direction, because
    /// `FR-CAT-013` carries a key on the table it names and this key names
    /// none.
    pub referenced_table: Option<Cow<'a, str>>,

    /// The key on the referenced table the foreign key points at, read from
    /// the unique-constraint name field, or [`None`] where the catalogue
    /// returned SQL `NULL` (`FR-CAT-056`).
    ///
    /// The field is declared nullable, `varchar(64)`, on all four series. It
    /// read `PRIMARY` on all fifteen keys of the fixture.
    pub referenced_key: Option<Cow<'a, str>>,

    /// The match option, carried verbatim. It read `NONE` on all fifteen, and
    /// no second value was observed.
    pub match_option: Cow<'a, str>,

    /// The `ON UPDATE` rule.
    pub on_update: ReferentialAction<'a>,

    /// The `ON DELETE` rule.
    pub on_delete: ReferentialAction<'a>,
}

/// A foreign key seen from the table it references (`FR-CAT-013`).
///
/// There is no second catalogue table for the incoming direction and no field
/// that states it: it is the outgoing rows of every other table, read from the
/// other end, by filtering the same key-column-usage table on the referenced
/// table name.
///
/// The referencing table **is** a field here, and it is not the restatement
/// `FR-CTX-021` forbids. `BR-CAT-005` excludes the field that names the object
/// a row hangs from only *where the model already carries it there*: a key in
/// [`Table::foreign_keys`](super::table::Table::foreign_keys) hangs from the
/// table carrying it, and a key in
/// [`Table::referenced_by`](super::table::Table::referenced_by) hangs from
/// another table entirely.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct IncomingForeignKey<'a> {
    /// The referencing table — the table the key is declared on.
    ///
    /// As with [`ForeignKey::referenced_table`], the model carries the name and
    /// the embedding of `FR-CTX-010` is the document's to apply.
    pub table: Cow<'a, str>,

    /// The key itself, exactly as the referencing table carries it.
    pub key: ForeignKey<'a>,
}

/// A copy that borrows nothing, for the render context of `FR-RND-023`.
impl ToStatic for ForeignKeyColumn<'_> {
    type Static = ForeignKeyColumn<'static>;

    fn to_static(&self) -> Self::Static {
        ForeignKeyColumn {
            column: self.column.to_static(),
            referenced_column: self.referenced_column.to_static(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ForeignKey, ForeignKeyColumn, IncomingForeignKey, ReferentialAction};
    use std::borrow::Cow;

    fn pair<'a>(column: &'a str, referenced: &'a str) -> ForeignKeyColumn<'a> {
        ForeignKeyColumn {
            column: Cow::Borrowed(column),
            referenced_column: Cow::Borrowed(referenced),
        }
    }

    fn key<'a>(name: &'a str, columns: Vec<ForeignKeyColumn<'a>>) -> ForeignKey<'a> {
        ForeignKey {
            name: Cow::Borrowed(name),
            columns,
            referenced_table: Some(Cow::Borrowed("consignment")),
            referenced_key: Some(Cow::Borrowed("PRIMARY")),
            match_option: Cow::Borrowed("NONE"),
            on_update: ReferentialAction::Cascade,
            on_delete: ReferentialAction::Restrict,
        }
    }

    #[test]
    fn fr_cat_045_the_two_column_lists_are_one_list_of_pairs_and_cannot_drift() {
        // FR-CAT-045 pairs the referencing and referenced columns
        // positionally. One list of pairs makes a length mismatch and a
        // reordering of one half unrepresentable.
        let composite = key(
            "fk_leg_consignment",
            vec![
                pair("consignment_id", "consignment_id"),
                pair("tenant_id", "tenant_id"),
            ],
        );

        assert_eq!(composite.columns.len(), 2);
        assert_eq!(composite.columns[0].column, "consignment_id");
        assert_eq!(composite.columns[0].referenced_column, "consignment_id");
    }

    #[test]
    fn fr_cat_045_the_four_reachable_rule_spellings_round_trip_and_set_default_is_not_one() {
        // FR-CAT-045 fixes four spellings as contract surface. FR-CAT-033
        // observed that `SET DEFAULT` is accepted, discarded silently, and
        // reported as `RESTRICT`, so no read can return it — which is why it
        // has no recorded variant to be written into.
        for action in [
            ReferentialAction::Cascade,
            ReferentialAction::NoAction,
            ReferentialAction::Restrict,
            ReferentialAction::SetNull,
        ] {
            assert_eq!(
                ReferentialAction::from_catalogue(action.name()),
                action.clone()
            );
            assert_eq!(action.recorded(), Some(action.name()));
        }

        assert_eq!(
            ReferentialAction::from_catalogue("SET DEFAULT").recorded(),
            None,
            "SET DEFAULT has no recorded variant, per FR-CAT-033"
        );
    }

    #[test]
    fn fr_cat_055_a_rule_outside_the_recorded_four_is_carried_and_is_not_a_refusal() {
        // FR-CAT-055: both rule fields are `varchar(64)` and never an `ENUM`
        // on any of the four series, so nothing the server declares closes the
        // set. A fifth spelling is carried as the catalogue wrote it — the
        // read does not fail, the key is not dropped, and the reading is exact
        // in case and in spacing.
        for outside in ["SET DEFAULT", "NO_ACTION", "cascade", ""] {
            let carried = ReferentialAction::from_catalogue(outside);

            assert_eq!(
                carried,
                ReferentialAction::Unrecorded(Cow::Borrowed(outside))
            );
            assert_eq!(carried.name(), outside);
            assert_eq!(carried.recorded(), None);
        }
    }

    #[test]
    fn fr_cat_055_a_rule_round_trips_through_the_document_whether_recorded_or_not() {
        // FR-CAT-055 with FR-SRV-031: a document written from a server newer
        // than the window carries what that server said, and the cache reads
        // it back unchanged.
        for action in [
            ReferentialAction::Cascade,
            ReferentialAction::NoAction,
            ReferentialAction::Restrict,
            ReferentialAction::SetNull,
            ReferentialAction::Unrecorded(Cow::Borrowed("SET DEFAULT")),
        ] {
            let written = serde_json::to_string(&action).expect("a rule serialises");
            let read: ReferentialAction<'_> =
                serde_json::from_str(&written).expect("a rule reads back");

            assert_eq!(written, format!("\"{}\"", action.name()));
            assert_eq!(read, action);
        }
    }

    #[test]
    fn fr_cat_056_a_key_that_names_no_table_and_no_unique_constraint_is_still_a_key() {
        // FR-CAT-056: both fields are declared nullable on all four series and
        // reach the model as `None` rather than as the empty string.
        // FR-CTX-006 requires such a key to be carried whole all the same —
        // its name, its columns and its rules are facts the catalogue did
        // return.
        let unresolved = ForeignKey {
            referenced_table: None,
            referenced_key: None,
            ..key("fk_leg_consignment", vec![pair("consignment_id", "")])
        };

        assert_eq!(unresolved.referenced_table, None);
        assert_eq!(unresolved.referenced_key, None);
        assert_eq!(unresolved.name, "fk_leg_consignment");
        assert_eq!(unresolved.columns.len(), 1);
        assert_eq!(unresolved.on_update, ReferentialAction::Cascade);
    }

    #[test]
    fn br_cat_005_the_outgoing_direction_does_not_name_its_own_table_and_the_incoming_one_does() {
        // BR-CAT-005 excludes the field naming the object a row hangs from
        // only where the model already carries it there. It does for an
        // outgoing key and it does not for an incoming one.
        let outgoing = key(
            "fk_leg_consignment",
            vec![pair("consignment_id", "consignment_id")],
        );
        let incoming = IncomingForeignKey {
            table: Cow::Borrowed("consignment_leg"),
            key: outgoing.clone(),
        };

        assert_eq!(incoming.table, "consignment_leg");
        assert_eq!(
            incoming.key.referenced_table,
            Some(Cow::Borrowed("consignment"))
        );
        assert_eq!(incoming.key, outgoing);
    }
}
