//! A view (`FR-CAT-007`, `FR-CAT-047`).
//!
//! Nine properties from a catalogue field list of eleven; the other two are the
//! catalogue and schema names, which are row identity.
//!
//! *A view carries no comment, and the absence is deliberate.* The row a view
//! has in the table catalogue carries the four-character literal `VIEW` in its
//! comment field — not the empty string and not SQL `NULL` — on all five views
//! of the fixture and all four series. `FR-CAT-040` refuses it: the value is a
//! constant the server writes rather than text an author supplied, `CREATE
//! VIEW` offers no clause that could put anything else there, and a template
//! could not tell it from an authored comment. There is no field for it here,
//! so it cannot be carried by accident.
//!
//! *There is no creation or alteration timestamp on a view either*, so the
//! volatile exclusion of `FR-CAT-024` has nothing to reach.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::restricted::Restricted;

/// One view (`FR-CAT-047`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct View<'a> {
    /// The view's name.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// The SQL definition, per `FR-CAT-007` and `FR-SCH-006`.
    ///
    /// It is the server's **rewritten** form — fully qualified and
    /// backtick-quoted — and not the text of the `CREATE VIEW`.
    pub definition: Cow<'a, str>,

    /// The check option, carried verbatim. `NONE`, `LOCAL` and `CASCADED` were
    /// all three observed.
    pub check_option: Cow<'a, str>,

    /// Whether the view is updatable, read from a field that returns `YES` or
    /// `NO`; both were observed.
    pub is_updatable: bool,

    /// The definer, which read one user across the fixture's five views.
    pub definer: Cow<'a, str>,

    /// The security type. `DEFINER` and `INVOKER` were both observed.
    pub security_type: Cow<'a, str>,

    /// The session character set, passed through verbatim per `FR-SRV-039`.
    pub character_set_client: Cow<'a, str>,

    /// The session collation, passed through verbatim per `FR-SRV-039`.
    ///
    /// This is one of the two fields the divergence register of `FR-SRV-036`
    /// holds: it differs between `10.11` and the other three series from
    /// identical DDL, because the servers' own default collations differ.
    pub collation_connection: Cow<'a, str>,

    /// The algorithm, which read `UNDEFINED` on all five views of the fixture;
    /// no second value was observed.
    pub algorithm: Cow<'a, str>,

    /// The properties of this view that could not be read, or [`None`] where
    /// it is complete (`FR-PRIV-005` … `FR-PRIV-007`).
    ///
    /// A view is one of the two object kinds a cross-check exists for: the
    /// rows of table type `VIEW` are counted against the readability of each
    /// definition, per `FR-PRIV-011` and `FR-PRIV-015`.
    ///
    /// `FR-PRIV-016` puts the marking on an incomplete object **only**, so the
    /// key is absent rather than `null` where the view is complete. This is the
    /// one exception `OD-18` admits to `FR-OUT-012`, and it is expressed the
    /// way that entry fixes.
    #[serde(borrow, default, skip_serializing_if = "Option::is_none")]
    pub restricted: Option<Restricted<'a>>,
}

#[cfg(test)]
mod tests {
    use super::View;
    use crate::model::restricted::Restricted;
    use std::borrow::Cow;

    fn view(restricted: Option<Restricted<'static>>) -> View<'static> {
        View {
            name: Cow::Borrowed("v_consignment_manifest"),
            definition: Cow::Borrowed("select `freight`.`consignment`.`reference` AS `reference`"),
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

    #[test]
    fn fr_priv_007_a_complete_view_carries_no_marking_and_an_incomplete_one_names_a_property() {
        // FR-PRIV-007: a complete object is not marked. FR-PRIV-016: where the
        // marking is present it names at least one property.
        assert_eq!(view(None).restricted, None);

        let incomplete = view(Restricted::new(vec![Cow::Borrowed("definition")]));
        let marking = incomplete
            .restricted
            .as_ref()
            .expect("the view was built incomplete");

        assert_eq!(marking.properties(), [Cow::Borrowed("definition")]);
    }

    #[test]
    fn fr_srv_039_the_session_collation_is_passed_through_exactly_as_the_server_returns_it() {
        // FR-SRV-039: no normalisation, substitution or adjustment. The two
        // values below are two different collations, not two spellings of one.
        let newer = view(None);
        let older = View {
            collation_connection: Cow::Borrowed("utf8mb4_general_ci"),
            ..view(None)
        };

        assert_eq!(newer.collation_connection, "utf8mb4_uca1400_ai_ci");
        assert_eq!(older.collation_connection, "utf8mb4_general_ci");
    }
}
