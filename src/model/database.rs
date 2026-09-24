//! The database, which is the root of the model (`FR-CTX-035`, `FR-CTX-036`).
//!
//! This is the second of the two objects that is **not a catalogue row** in the
//! sense the rest of the model is. The schema catalogue table returns six
//! columns and no more, identically on all four series, and three of them
//! become fields here. Of the other three: the catalogue-name field reads `def`
//! for every schema and locates the row rather than describing the database,
//! per `BR-CAT-005`; the SQL-path field is SQL `NULL` for every schema on every
//! series, so nothing has been observed for the model to carry; and the
//! schema-comment field is the empty string for every schema on every series
//! and the fixture declares none, so admitting it would need an observation
//! rather than a decision.
//!
//! Beside them stand the `server` object of `FR-CTX-031` and the three
//! collections of `FR-CTX-035` — the three object kinds
//! [catalogue coverage](super) covers, under the names every command of the
//! first arm already presents them by.
//!
//! *`charset` and `collation` are passed through verbatim, per `FR-SRV-039`,
//! and this is one of the two places the divergence register of `FR-SRV-036`
//! reaches.* A database created without an explicit collation yields a
//! different value from two supported servers — `utf8mb4_general_ci` on
//! `10.11` and `utf8mb4_uca1400_ai_ci` on the other three — and `FR-SRV-026`
//! excepts it for that reason. Normalising either onto the other would discard
//! the information the field exists to carry: they are two different
//! collations, not two spellings of one.

use std::borrow::Cow;

use super::routine::Routine;
use super::server::Server;
use super::table::Table;
use super::view::View;

/// The database a model was read from (`FR-CTX-036`).
///
/// The three collections are `Vec`s rather than maps keyed by name.
/// `FR-CTX-005` rejected the maps: ordering would then exist only in the
/// emitter and could not be read back from the document, and `NFR-DET-002`
/// would have no observable subject.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Database<'a> {
    /// The schema's name.
    pub name: Cow<'a, str>,

    /// The schema's default character set, passed through verbatim per
    /// `FR-SRV-039`.
    pub charset: Cow<'a, str>,

    /// The schema's default collation, passed through verbatim per
    /// `FR-SRV-039`.
    pub collation: Cow<'a, str>,

    /// The server the read was made against (`FR-CTX-031`).
    ///
    /// A cache-served model carries the server the read was made against
    /// rather than any server reachable now, which is what `FR-CDOC-009`
    /// already signals about every other field in it.
    pub server: Server<'a>,

    /// The tables of `FR-CAT-001`.
    pub tables: Vec<Table<'a>>,

    /// The views of `FR-CAT-007`.
    pub views: Vec<View<'a>>,

    /// The routines of `FR-CAT-008` — procedures and functions together.
    pub routines: Vec<Routine<'a>>,
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::model::server::{Server, Standing};
    use crate::model::table::{Table, TableParts, TableType};
    use std::borrow::Cow;

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
            tables: Vec::new(),
            views: Vec::new(),
            routines: Vec::new(),
        }
    }

    #[test]
    fn fr_ctx_036_the_database_carries_the_three_metadata_fields_the_requirement_fixes() {
        // FR-CTX-036: exactly three, and the values observed for the fixture.
        let freight = database();

        assert_eq!(freight.name, "freight");
        assert_eq!(freight.charset, "utf8mb4");
        assert_eq!(freight.collation, "utf8mb4_unicode_520_ci");
    }

    #[test]
    fn fr_ctx_035_the_three_collections_stand_beside_the_server_and_are_empty_rather_than_absent() {
        // FR-CTX-035 with FR-CTX-004: an empty collection is `[]`, never null
        // and never omitted, so a consumer may test it for emptiness without
        // first testing it for nullity.
        let freight = database();

        assert!(freight.tables.is_empty());
        assert!(freight.views.is_empty());
        assert!(freight.routines.is_empty());
        assert_eq!(freight.server.series(), "11.4");
    }

    #[test]
    fn fr_ctx_035_a_table_that_assembled_is_carried_in_the_tables_collection() {
        let freight = Database {
            tables: vec![
                Table::assemble(TableParts::new(
                    Cow::Borrowed("consignment"),
                    TableType::Base,
                ))
                .expect("a table with no keys satisfies FR-CAT-044 vacuously"),
            ],
            ..database()
        };

        assert_eq!(freight.tables.len(), 1);
        assert_eq!(freight.tables[0].name(), "consignment");
    }
}
