//! The ordering of every collection the document carries (`NFR-DET-002`,
//! `FR-CTX-003`, `FR-PRIV-016`).
//!
//! The requirement is one default and a table of six exceptions:
//!
//! | Collection | Order |
//! |---|---|
//! | A table's columns | Ordinal position |
//! | An index's columns | The order the catalogue states |
//! | A primary key's columns | The order the catalogue states |
//! | A foreign key's columns, and the referenced columns paired with them | The order the catalogue states |
//! | An `ENUM` or `SET` member list | The order the catalogue states |
//! | A routine's parameters | Declaration order |
//! | Every other collection | Name, ascending, byte-wise |
//!
//! **Three of the six are silent corruptions if the default rule reaches
//! them**, which is what the seventh-edition amendment records: a primary key
//! sorted by column name is a *different key*; a foreign key's two column lists
//! are paired by position, so sorting either pairs each column with the wrong
//! counterpart; and an `ENUM` member's position **is** the ordinal it is stored
//! as, so sorting the list renumbers every member and a generator emits every
//! discriminant wrongly, at exit `0`.
//!
//! *The default rule is therefore kept away from them by the type system, not
//! by review.* [`by_name`] and [`sort_by_name`] are bounded by [`Named`], and
//! [`Named`] is implemented **only** for the member types of the collections
//! the default rule governs. A column, an index column, a foreign-key column
//! pair and a routine parameter each have a name and none of them implements
//! this trait, so `by_name(table.columns())` does not compile. The other two
//! exceptions — an index's columns and an `ENUM` member list — are fields of
//! types that reach the document through their own derived serialisation, so
//! nothing in this module can reach them at all.
//!
//! An excepted collection is carried through [`as_given`], which states at the
//! call site which of the two rules that collection falls under — the device
//! [`Order`](crate::output::Order) already uses for the `text` listings.
//!
//! **The comparison is byte-wise and is never locale-aware.** [`compare`] is
//! the one comparator; every ordering in the document is derived from it, so
//! two runs on two machines in two locales produce the same document. The
//! accepted cost `NFR-DET-002` records is that `Orders` precedes `customers`
//! and `_internal` follows `Zebra`.

use std::borrow::Cow;
use std::cmp::Ordering;

use crate::model::check_constraint::CheckConstraint;
use crate::model::index::Index;
use crate::model::routine::Routine;
use crate::model::table::Table;
use crate::model::trigger::Trigger;
use crate::model::view::View;

/// A member of a collection the default rule of `NFR-DET-002` governs.
///
/// Implementing it is the whole of the decision that a collection of this type
/// is ordered by name. The six excepted collections have no implementation, and
/// that absence is what keeps the default rule away from them.
pub(crate) trait Named {
    /// The name the default rule orders by.
    fn name(&self) -> &str;
}

impl Named for Cow<'_, str> {
    /// A bare string is its own name.
    ///
    /// This is the member type of the two cut collections of `FR-CTX-008` and
    /// of the `restricted` marking of `FR-PRIV-016`.
    fn name(&self) -> &str {
        self
    }
}

impl Named for Table<'_> {
    /// The inherent accessor, named through the type so that the path resolves
    /// to it and not to this method.
    fn name(&self) -> &str {
        Table::name(self)
    }
}

impl Named for Index<'_> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Trigger<'_> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for CheckConstraint<'_> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for View<'_> {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Named for Routine<'_> {
    fn name(&self) -> &str {
        &self.name
    }
}

/// The one comparator of `NFR-DET-002`: byte by byte, ascending.
///
/// It is `str`'s own ordering, written over the bytes so that the requirement's
/// *compared byte by byte* is visible at the point it is satisfied rather than
/// inferred from a standard-library guarantee.
pub(crate) fn compare(left: &str, right: &str) -> Ordering {
    left.as_bytes().cmp(right.as_bytes())
}

/// Orders a collection the caller has already built, in place.
///
/// The sort is stable, so two members of one name keep the order they were
/// built in and the result does not depend on the sort's internals.
pub(crate) fn sort_by_name<T: Named>(members: &mut [T]) {
    members.sort_by(|left, right| compare(left.name(), right.name()));
}

/// Carries a collection into the document under the default rule.
///
/// The members are **borrowed** where they already stand in that order, which
/// is the case a catalogue read with an `ORDER BY` produces and the case a
/// document read back produces; a copy is made only where a member has to move.
/// The answer is therefore correct whatever order the model arrived in, and
/// costs nothing where the model arrived ordered.
pub(crate) fn by_name<T: Named + Clone>(members: &[T]) -> Cow<'_, [T]> {
    if members
        .windows(2)
        .all(|pair| compare(pair[0].name(), pair[1].name()).is_le())
    {
        return Cow::Borrowed(members);
    }

    let mut ordered = members.to_vec();
    sort_by_name(&mut ordered);
    Cow::Owned(ordered)
}

/// Pointers to a collection's members, under the default rule.
///
/// It is [`by_name`] for a collection whose members are **transformed** on the
/// way into the document rather than carried into it: the ordered copy of
/// [`by_name`] would own the members, and the values built from it could then
/// borrow no further than that copy.
///
/// The result is also what a lookup by name binary-searches, which is why the
/// caller keeps it rather than iterating it once.
pub(crate) fn pointers_by_name<T: Named>(members: &[T]) -> Vec<&T> {
    let mut pointers: Vec<&T> = members.iter().collect();
    pointers.sort_by(|left, right| compare(left.name(), right.name()));
    pointers
}

/// Carries a collection into the document in the order it was given.
///
/// This is the call site of an exception of `NFR-DET-002`, and it is a function
/// rather than a bare [`Cow::Borrowed`] so that every excepted collection says
/// so where it is built. It is bounded by nothing, because an excepted
/// collection's member type deliberately implements no [`Named`].
pub(crate) fn as_given<T: Clone>(members: &[T]) -> Cow<'_, [T]> {
    Cow::Borrowed(members)
}

#[cfg(test)]
mod tests {
    use super::{Named, as_given, by_name, compare, pointers_by_name, sort_by_name};
    use std::borrow::Cow;

    /// A member with a name and a payload, so a test can see which value moved.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Member {
        name: &'static str,
        payload: u8,
    }

    impl Named for Member {
        fn name(&self) -> &str {
            self.name
        }
    }

    fn member(name: &'static str, payload: u8) -> Member {
        Member { name, payload }
    }

    #[test]
    fn nfr_det_002_the_comparison_is_byte_wise_and_not_what_a_collation_would_give() {
        // NFR-DET-002's accepted cost: `Orders` precedes `customers` because
        // every upper-case ASCII letter sorts below every lower-case one, and
        // `_internal` follows `Zebra` for the same reason. A locale-aware
        // comparison gives the opposite answer for both.
        assert!(compare("Orders", "customers").is_lt());
        assert!(compare("Zebra", "_internal").is_lt());
        assert!(compare("consignment", "consignment_leg").is_lt());
        assert!(compare("tariff", "tariff").is_eq());
    }

    #[test]
    fn a_collection_already_in_order_is_borrowed_rather_than_copied() {
        let members = [member("carrier", 1), member("consignment", 2)];

        assert!(matches!(by_name(&members), Cow::Borrowed(_)));
    }

    #[test]
    fn nfr_det_002_a_collection_out_of_order_is_copied_and_ordered() {
        let members = [member("vessel", 1), member("carrier", 2)];
        let ordered = by_name(&members);

        assert!(matches!(ordered, Cow::Owned(_)));
        assert_eq!(
            ordered.as_ref(),
            [member("carrier", 2), member("vessel", 1)]
        );
    }

    #[test]
    fn nfr_det_002_two_members_of_one_name_keep_the_order_they_were_built_in() {
        // The sort is stable, so the result does not depend on its internals.
        let mut members = [member("leg", 3), member("carrier", 1), member("leg", 4)];
        sort_by_name(&mut members);

        assert_eq!(
            members,
            [member("carrier", 1), member("leg", 3), member("leg", 4)]
        );
    }

    #[test]
    fn nfr_det_002_an_excepted_collection_is_carried_in_the_order_it_was_given() {
        // NFR-DET-002's six exceptions: the order is the meaning, and this is
        // the call the collections that carry one are built through.
        let members = [member("vessel_imo", 1), member("leg_sequence", 2)];

        assert_eq!(as_given(&members).as_ref(), members);
        assert!(matches!(as_given(&members), Cow::Borrowed(_)));
    }

    #[test]
    fn nfr_det_002_pointers_are_ordered_without_the_members_moving() {
        let members = [member("vessel", 1), member("carrier", 2)];
        let ordered = pointers_by_name(&members);

        assert_eq!(ordered[0].name, "carrier");
        assert_eq!(ordered[1].name, "vessel");
        assert!(std::ptr::eq(ordered[0], &members[1]));
    }

    #[test]
    fn fr_ctx_008_a_bare_string_is_its_own_name() {
        // The member type of the two cut collections of FR-CTX-008 and of the
        // `restricted` marking of FR-PRIV-016.
        let mut names = [
            Cow::Borrowed("vessel"),
            Cow::Owned("carrier".to_owned()),
            Cow::Borrowed("Berth"),
        ];
        sort_by_name(&mut names);

        assert_eq!(names, ["Berth", "carrier", "vessel"]);
    }
}
