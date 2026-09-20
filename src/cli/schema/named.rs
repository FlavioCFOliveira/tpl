//! Naming one object: the qualified routine form, the lookup, the `66` a name
//! that reaches nothing produces, and the `77` a short object owes.
//!
//! Three commands of the first arm name an object positionally, per
//! `FR-SCH-005`, and two of `tpl cache` name one by flag, per `FR-CACHE-024`.
//! Everything they share is here.
//!
//! | Rule | Requirement |
//! |---|---|
//! | `procedure:<name>` and `function:<name>` are accepted beside a bare name | `FR-SCH-008` |
//! | The prefix is matched **as written**, in lower case, and any other case is `64` | `FR-SCH-008` |
//! | A bare name matching both a procedure and a function is `64`, naming both in the qualified form | `FR-SCH-010` |
//! | A name that reaches no object is `66`, with a nearest-match suggestion over the objects of that kind that do exist | `FR-SCH-010`, `FR-ERR-019` |
//! | A named object that came back short is `77`, and is not returned in part | `FR-PRIV-003`, `FR-PRIV-004` |
//!
//! **The prefix is decided from the token alone**, so it precedes every
//! catalogue read: `FR-SCH-008` places the condition at step 1 of `FR-ERR-006`,
//! and [`routine_token`] is called before anything opens a connection.
//!
//! **The suggestion is selected here and composed in `diagnostics`.**
//! `FR-ERR-021`'s populations belong to the components that own them, and the
//! population of a catalogue object is the collection the read produced; the
//! line the caller reads is composed by [`crate::diagnostics`] from the names
//! the error carries.

use crate::diagnostics::suggest::{self, Population};
use crate::error::{CatalogueObjectKind, Error};
use crate::model::document::DatabaseDocument;
use crate::model::document::shape::TableDocument;
use crate::model::routine::{Routine, RoutineKind};
use crate::model::view::View;

/// The separator between a qualifying prefix and the routine name
/// (`FR-SCH-008`).
const QUALIFIER: char = ':';

/// The prefix `FR-SCH-008` fixes for a stored procedure, in the lower case it
/// fixes it in.
const PROCEDURE: &str = "procedure";

/// The prefix `FR-SCH-008` fixes for a stored function.
const FUNCTION: &str = "function";

/// What a token naming one routine resolved to (`FR-SCH-008`).
///
/// It is [`Clone`] and not [`Copy`], because [`RoutineKind`] carries the
/// catalogue's own string for a kind outside the recorded two, per
/// `FR-CAT-055`. Only the two recorded kinds are ever built here: a qualified
/// token carries one of the two prefixes `FR-SCH-008` admits and no third.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Wanted<'a> {
    /// A bare name, which may denote a procedure **and** a function — the
    /// ambiguity `FR-SCH-010` refuses.
    Bare(&'a str),

    /// A name qualified by one of the two prefixes.
    Qualified(RoutineKind<'a>, &'a str),
}

impl<'a> Wanted<'a> {
    /// The routine name, without any qualifying prefix.
    const fn name(&self) -> &'a str {
        match self {
            Self::Bare(name) | Self::Qualified(_, name) => name,
        }
    }
}

/// Resolves a token that names one routine (`FR-SCH-008`).
///
/// `invocation` is the command path below `tpl` the token was given to, and it
/// is a literal of `FR-ERR-022` — the hint of a refusal is the same invocation
/// with the prefix corrected, so the path travels on the condition.
///
/// The segment before the first colon is folded over ASCII `A-Z` and `a-z`
/// alone, exactly as `FR-SCH-014` folds: a segment that folds to neither prefix
/// is not a prefix at all and the whole token is a bare name, colon included.
///
/// # Errors
///
/// Returns [`Error::RoutinePrefixNotLowerCase`] — `64` — where the segment
/// folds to `procedure` or `function` and the token does not carry it in lower
/// case.
pub(crate) fn routine_token<'t>(
    token: &'t str,
    invocation: &'static str,
) -> Result<Wanted<'t>, Error> {
    let Some((prefix, name)) = token.split_once(QUALIFIER) else {
        return Ok(Wanted::Bare(token));
    };

    let kind = match prefix.to_ascii_lowercase().as_str() {
        PROCEDURE => RoutineKind::Procedure,
        FUNCTION => RoutineKind::Function,
        // A segment that folds to neither is not a prefix, so the token is a
        // bare name — one that happens to carry a colon, which a catalogue
        // name may.
        _ => return Ok(Wanted::Bare(token)),
    };
    // A kind with no lower-case spelling is not a prefix `FR-SCH-008` admits,
    // so the token is a bare name on the same terms as the arm above. The arm
    // is unreachable from the match that produced `kind` — both of its arms
    // yield a recorded kind — and it is written as a degradation rather than
    // as an `expect` so that this module carries no panic.
    let Some(lower) = lower(&kind) else {
        return Ok(Wanted::Bare(token));
    };

    if prefix != lower {
        return Err(Error::RoutinePrefixNotLowerCase {
            token: token.to_owned(),
            prefix: lower,
            name: name.to_owned(),
            invocation,
        });
    }

    Ok(Wanted::Qualified(kind, name))
}

/// The lower-case spelling of a routine kind (`FR-SCH-008`), or [`None`] where
/// the kind has none.
///
/// It is the spelling the cache composes a path from, per `FR-CDOC-014`, and
/// is read from the one place that states it so that the command line and the
/// path cannot drift apart. [`None`] is the kind outside the recorded two that
/// `FR-CAT-055` carries and that neither this requirement nor `FR-CDOC-014`
/// gives a prefix.
fn lower(kind: &RoutineKind<'_>) -> Option<&'static str> {
    crate::cache::paths::lower(kind)
}

/// Where a named read was made, for the two conditions whose `cause` names the
/// population (`FR-ERR-034`, the `66` and `64` rows).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Sought<'a> {
    /// The database entry of `.tpl/.cfg` the read was made through.
    pub(crate) entry: &'a str,

    /// The server-side database the object was sought in (`FR-CONF-041`).
    pub(crate) database: &'a str,
}

/// The table `name` names (`FR-SCH-010`, `FR-PRIV-003`).
///
/// # Errors
///
/// Returns [`Error::CatalogueObjectNotFound`] — `66` — where the database holds
/// no such table, carrying the nearest matches among the tables it does hold;
/// and [`Error::PropertyNotReadable`] — `77` — where the table came back short,
/// per `FR-PRIV-004`.
pub(crate) fn table<'a, 'd>(
    document: &'a DatabaseDocument<'d>,
    name: &str,
    at: Sought<'_>,
) -> Result<&'a TableDocument<'d>, Error> {
    let found = document
        .tables
        .iter()
        .find(|table| table.name == name)
        .ok_or_else(|| {
            absent(
                CatalogueObjectKind::Table,
                name,
                at,
                document.tables.iter().map(|table| table.name.as_ref()),
            )
        })?;

    crate::mariadb::catalogue::completeness::of_table(found)?;

    Ok(found)
}

/// The view `name` names (`FR-SCH-010`, `FR-PRIV-003`).
///
/// # Errors
///
/// Returns what [`table`] returns, for the view population.
pub(crate) fn view<'a, 'd>(
    document: &'a DatabaseDocument<'d>,
    name: &str,
    at: Sought<'_>,
) -> Result<&'a View<'d>, Error> {
    let found = document
        .views
        .iter()
        .find(|view| view.name == name)
        .ok_or_else(|| {
            absent(
                CatalogueObjectKind::View,
                name,
                at,
                document.views.iter().map(|view| view.name.as_ref()),
            )
        })?;

    crate::mariadb::catalogue::completeness::of_view(found)?;

    Ok(found)
}

/// The routine `wanted` names (`FR-SCH-008`, `FR-SCH-010`, `FR-PRIV-003`).
///
/// A bare name is matched against both namespaces, because procedures and
/// functions occupy distinct ones on the server: two matches are the ambiguity
/// `FR-SCH-010` refuses with `64`, one is the answer, and none is the `66` every
/// other kind produces.
///
/// # Errors
///
/// Returns [`Error::AmbiguousRoutineName`] — `64` — where a bare name matches
/// both kinds; [`Error::CatalogueObjectNotFound`] — `66` — where it matches
/// neither; and [`Error::PropertyNotReadable`] — `77` — where the routine came
/// back short.
pub(crate) fn routine<'a, 'd>(
    document: &'a DatabaseDocument<'d>,
    wanted: &Wanted<'_>,
    at: Sought<'_>,
    invocation: &'static str,
) -> Result<&'a Routine<'d>, Error> {
    let name = wanted.name();
    let mut matched = document.routines.iter().filter(|routine| {
        routine.name == name
            && match wanted {
                Wanted::Bare(_) => true,
                // The comparison is over the whole value, so a kind outside
                // the recorded two never matches a qualified token: no such
                // token can be built. That is `FR-CAT-055`'s consequence
                // reaching the lookup, and the bare form above is what still
                // reaches such a routine.
                Wanted::Qualified(kind, _) => &routine.kind == kind,
            }
    });

    let found = matched.next().ok_or_else(|| {
        absent(
            CatalogueObjectKind::Routine,
            name,
            at,
            // The population is every routine of the database, whichever kind
            // the token asked for: a caller that wrote `procedure:calc_vat`
            // for a function is best served by being shown `calc_vat`.
            document
                .routines
                .iter()
                .map(|routine| routine.name.as_ref()),
        )
    })?;

    if matched.next().is_some() {
        return Err(Error::AmbiguousRoutineName {
            name: name.to_owned(),
            entry: at.entry.to_owned(),
            database: at.database.to_owned(),
            invocation,
        });
    }

    crate::mariadb::catalogue::completeness::of_routine(found)?;

    Ok(found)
}

/// The `66` of `FR-SCH-010`, with the nearest matches `FR-ERR-019` selects.
///
/// The candidates are the names of the objects of that kind the database does
/// hold, which is the population `FR-SCH-010` names. `FR-ERR-020` leaves the
/// suggestion out where nothing qualified, and `FR-ERR-023` drops a candidate
/// the character set of `FR-ERR-022` refuses — both of which the composition in
/// [`crate::diagnostics`] applies to what this carries.
fn absent<'n, C>(kind: CatalogueObjectKind, name: &str, at: Sought<'_>, candidates: C) -> Error
where
    C: IntoIterator<Item = &'n str>,
{
    let nearest = suggest::suggestions(name, candidates, Population::Names);

    Error::CatalogueObjectNotFound {
        kind,
        name: name.to_owned(),
        entry: at.entry.to_owned(),
        database: at.database.to_owned(),
        nearest: nearest.names().map(str::to_owned).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Sought, Wanted, routine_token};
    use crate::error::Error;
    use crate::model::database::Database;
    use crate::model::document::{context, fixture};
    use crate::model::routine::RoutineKind;

    /// The invocation every token below was given to.
    const INVOCATION: &str = "schema routine";

    #[test]
    fn fr_sch_008_the_two_qualified_forms_are_accepted_beside_a_bare_name() {
        assert_eq!(
            routine_token("calc_vat", INVOCATION).expect("a bare name"),
            Wanted::Bare("calc_vat")
        );
        assert_eq!(
            routine_token("procedure:calc_vat", INVOCATION).expect("a qualified name"),
            Wanted::Qualified(RoutineKind::Procedure, "calc_vat")
        );
        assert_eq!(
            routine_token("function:calc_vat", INVOCATION).expect("a qualified name"),
            Wanted::Qualified(RoutineKind::Function, "calc_vat")
        );
    }

    #[test]
    fn fr_sch_008_a_prefix_in_any_other_case_is_refused_rather_than_read_as_a_bare_name() {
        // FR-SCH-008 as the twenty-fifth edition amended it: the composition a
        // caller makes from `kind` — which FR-CAT-016 fixes in upper case —
        // produces exactly this token, and it is 64 rather than a bare name
        // refused with 66 over a population that cannot hold it.
        for written in [
            "PROCEDURE:calc_vat",
            "Procedure:calc_vat",
            "pRoCeDuRe:calc_vat",
            "FUNCTION:calc_vat",
            "Function:calc_vat",
        ] {
            let refused = routine_token(written, INVOCATION).expect_err("the case is refused");

            assert_eq!(refused.exit_code(), 64, "{written}");

            let Error::RoutinePrefixNotLowerCase { token, name, .. } = &refused else {
                panic!("{written} was refused as {refused}");
            };

            assert_eq!(token, written, "the token is reproduced as written");
            assert_eq!(name, "calc_vat");
        }
    }

    #[test]
    fn fr_sch_008_the_refusal_carries_the_same_invocation_with_the_prefix_corrected() {
        // FR-SCH-008: the hint carries the same invocation in lower case, per
        // FR-ERR-009.
        let refused =
            routine_token("PROCEDURE:calc_vat", INVOCATION).expect_err("the case is refused");
        let rendered = crate::diagnostics::rendered(&refused);

        assert!(
            rendered.contains("tpl schema routine procedure:calc_vat"),
            "{rendered}"
        );
    }

    #[test]
    fn fr_err_023_a_routine_name_outside_the_character_set_leaves_the_prefix_advice_alone() {
        // FR-ERR-023: a candidate the character set refuses is presented in no
        // form at all, so the runnable command is dropped and the generic
        // advice stands.
        let refused =
            routine_token("PROCEDURE:calc vat; drop", INVOCATION).expect_err("the case is refused");
        let rendered = crate::diagnostics::rendered(&refused);
        let hint = rendered
            .lines()
            .find(|line| line.starts_with("hint:"))
            .expect("FR-ERR-008 fixes four labelled lines");

        // The `cause` names the token as written, which FR-SCH-008 obliges and
        // FR-ERR-024 escapes; the **hint** is the runnable command, and it is
        // the line FR-ERR-023 keeps the name out of.
        assert!(rendered.contains("calc vat; drop"), "{rendered}");
        assert!(!hint.contains("drop"), "{hint}");
        assert!(hint.contains("lower case"), "{hint}");
    }

    #[test]
    fn fr_sch_008_a_segment_that_folds_to_neither_prefix_is_part_of_a_bare_name() {
        // A catalogue name may carry a colon, and only the two spellings this
        // requirement fixes are read as a prefix.
        for written in ["odd:name", ":leading", "trailing:", "a:b:c"] {
            assert_eq!(
                routine_token(written, INVOCATION).expect("not a qualified form"),
                Wanted::Bare(written),
                "{written}"
            );
        }
    }

    #[test]
    fn fr_sch_008_the_prefix_and_the_cached_path_carry_one_spelling() {
        // FR-CDOC-014 and FR-SCH-008 are the same rule at two layers, and the
        // spelling is read from one place so the two cannot drift.
        assert_eq!(super::lower(&RoutineKind::Procedure), Some("procedure"));
        assert_eq!(super::lower(&RoutineKind::Function), Some("function"));
    }

    /// Where every lookup below was made.
    fn at() -> Sought<'static> {
        Sought {
            entry: "shop",
            database: "freight",
        }
    }

    #[test]
    fn fr_err_037_a_suggestion_of_two_candidates_is_written_as_one_question() {
        // FR-ERR-037: all of them inside the one `did you mean` question, each
        // between single quotation marks, the last pair separated by ` or `,
        // in the order FR-ERR-019 fixes. The population is two views one
        // supplied name is within a distance of one of, which the reference
        // database of `scripts/mariadb/` does not happen to hold.
        let model = Database {
            views: vec![fixture::view("v_sales"), fixture::view("v_sale")],
            ..fixture::database_of(Vec::new())
        };
        let document = context(&model).expect("the model carries every reference");

        let refused = super::view(&document, "v_sals", at()).expect_err("no such view");
        let rendered = crate::diagnostics::rendered(&refused);

        assert_eq!(refused.exit_code(), 66);
        assert!(
            rendered.contains(
                "hint:  did you mean 'v_sale' or 'v_sales'? list the available views with: \
                 tpl -d shop schema views"
            ),
            "{rendered}"
        );
    }

    #[test]
    fn fr_sch_010_a_bare_name_matching_both_kinds_is_refused_rather_than_resolved() {
        // FR-SCH-010: the system resolves it in favour of neither, and the
        // `cause` names both candidates in the qualified form of FR-SCH-008.
        // The fixture's routine carries the `restricted` marking of
        // `FR-PRIV-017`, and this body is about the ambiguity rather than the
        // completeness verdict, so both are complete here.
        let mut procedure = fixture::routine("calc_vat");
        procedure.kind = RoutineKind::Procedure;
        procedure.restricted = None;
        let mut function = fixture::routine("calc_vat");
        function.kind = RoutineKind::Function;
        function.restricted = None;

        let model = Database {
            routines: vec![procedure, function],
            ..fixture::database_of(Vec::new())
        };
        let document = context(&model).expect("the model carries every reference");

        let refused = super::routine(&document, &Wanted::Bare("calc_vat"), at(), INVOCATION)
            .expect_err("the bare name matches both");
        let rendered = crate::diagnostics::rendered(&refused);

        assert_eq!(refused.exit_code(), 64);
        assert!(rendered.contains("'procedure:calc_vat'"), "{rendered}");
        assert!(rendered.contains("'function:calc_vat'"), "{rendered}");
        assert!(
            rendered.contains("tpl schema routine procedure:calc_vat"),
            "{rendered}"
        );

        // And the qualified form of either reaches its own object, which is
        // the whole point of the disambiguator.
        for (written, kind) in [
            ("procedure:calc_vat", RoutineKind::Procedure),
            ("function:calc_vat", RoutineKind::Function),
        ] {
            let wanted = routine_token(written, INVOCATION).expect("a qualified name");
            let found =
                super::routine(&document, &wanted, at(), INVOCATION).expect("the object is there");

            assert_eq!(found.kind, kind, "{written}");
        }
    }

    #[test]
    fn fr_priv_003_a_named_object_that_is_marked_is_refused_rather_than_returned_in_part() {
        // FR-PRIV-003 and FR-PRIV-004, over the document's own marking: the
        // verdict reads the same field whichever source served the read.
        let model = Database {
            views: vec![crate::model::view::View {
                restricted: crate::model::restricted::Restricted::new(vec![
                    std::borrow::Cow::Borrowed("definition"),
                ]),
                ..fixture::view("v_sales")
            }],
            ..fixture::database_of(Vec::new())
        };
        let document = context(&model).expect("the model carries every reference");

        let refused = super::view(&document, "v_sales", at()).expect_err("the view is marked");

        assert_eq!(refused.exit_code(), 77);
    }
}
