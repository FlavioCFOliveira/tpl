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
//! unsupported, so [`ReferentialAction`] has four variants: a generator written
//! against a fifth would carry a branch that never executes.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// The referential action of `ON UPDATE` and `ON DELETE` (`FR-CAT-045`).
///
/// The four spellings are contract surface, upper case, with a single space in
/// `NO ACTION`. `SET DEFAULT` is not among them, per `FR-CAT-033`, and has no
/// variant to be written into.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReferentialAction {
    /// The change is applied to the referencing rows.
    #[serde(rename = "CASCADE")]
    Cascade,

    /// The change is refused, by the standard's spelling.
    #[serde(rename = "NO ACTION")]
    NoAction,

    /// The change is refused, by MariaDB's own spelling.
    #[serde(rename = "RESTRICT")]
    Restrict,

    /// The referencing columns are set to `NULL`.
    #[serde(rename = "SET NULL")]
    SetNull,
}

impl ReferentialAction {
    /// Reads the action from the update-rule or delete-rule field.
    ///
    /// [`None`] for any other value. The four the catalogue was observed to
    /// return are the four variants, so a [`None`] here is a value no read of a
    /// supported series has produced.
    #[must_use]
    pub fn from_catalogue(field: &str) -> Option<Self> {
        match field {
            "CASCADE" => Some(Self::Cascade),
            "NO ACTION" => Some(Self::NoAction),
            "RESTRICT" => Some(Self::Restrict),
            "SET NULL" => Some(Self::SetNull),
            _ => None,
        }
    }

    /// The spelling `FR-CAT-045` fixes, which is what the document carries.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cascade => "CASCADE",
            Self::NoAction => "NO ACTION",
            Self::Restrict => "RESTRICT",
            Self::SetNull => "SET NULL",
        }
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

    /// The referenced table — **the table only**, without a schema.
    ///
    /// The model carries the name. The one-level-deep embedding of
    /// `FR-CTX-006` is a rule of the document, applied where the document is
    /// shaped, and it resolves this name against the model it was built from.
    pub referenced_table: Cow<'a, str>,

    /// The key on the referenced table the foreign key points at, read from
    /// the unique-constraint name field. It read `PRIMARY` on all fifteen keys
    /// of the fixture.
    pub referenced_key: Cow<'a, str>,

    /// The match option, carried verbatim. It read `NONE` on all fifteen, and
    /// no second value was observed.
    pub match_option: Cow<'a, str>,

    /// The `ON UPDATE` rule.
    pub on_update: ReferentialAction,

    /// The `ON DELETE` rule.
    pub on_delete: ReferentialAction,
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
            referenced_table: Cow::Borrowed("consignment"),
            referenced_key: Cow::Borrowed("PRIMARY"),
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
        // reported as `RESTRICT`, so no read can return it.
        for action in [
            ReferentialAction::Cascade,
            ReferentialAction::NoAction,
            ReferentialAction::Restrict,
            ReferentialAction::SetNull,
        ] {
            assert_eq!(
                ReferentialAction::from_catalogue(action.name()),
                Some(action)
            );
        }

        assert_eq!(ReferentialAction::from_catalogue("SET DEFAULT"), None);
        assert_eq!(ReferentialAction::from_catalogue("NO_ACTION"), None);
        assert_eq!(ReferentialAction::from_catalogue("cascade"), None);
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
        assert_eq!(incoming.key.referenced_table, "consignment");
        assert_eq!(incoming.key, outgoing);
    }
}
