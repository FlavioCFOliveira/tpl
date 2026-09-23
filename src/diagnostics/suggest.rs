//! Nearest-match selection: the distance, the ordering, the cap, and the
//! filter that decides which candidate may be presented at all.
//!
//! `FR-ERR-019` offers at most three suggestions drawn from names within an
//! edit distance of two, ordered by distance and then by name; `FR-ERR-020`
//! omits the suggestion rather than weaken it where nothing qualifies; and
//! `FR-ERR-021` applies the rule to eight populations. The **populations are
//! not here**: each belongs to the component that owns it — object names to
//! `mariadb/`, template names to `render/`, entry and key names to
//! `project/config.rs`, command and flag names to `cli/` — and this module owns
//! only the selection made over one of them.
//!
//! The **measure** is `FR-ERR-039`'s: the **restricted** Damerau-Levenshtein
//! distance, optimal string alignment, in which a transposition of two adjacent
//! characters costs one — so `ordres` is one step from `orders` where plain
//! Levenshtein reports two. The member and not merely the family is named there
//! because the two forms disagree inside the threshold `FR-ERR-019` fixes, so
//! the choice decides the candidate **set** and not only its order.
//!
//! The **computation** is `OD-20`'s: hand-rolled in this crate, with no
//! dependency, over a rolling window of three rows. `BR-PERF-004` makes this a
//! budgeted path — a `66` over `WL-001` compares against 200 names — so the
//! implementation reuses one scratch buffer across every comparison of a run
//! and refuses a candidate on its length before measuring it.
//!
//! [`Population`] carries `FR-ERR-022` as the twentieth edition amended it. A
//! spelling this specification enumerates — a command or an alias of the
//! command tree, a flag a node declares, a key of `FR-CONF-002` — is a
//! **literal**, and the character set does not govern it; every other value is
//! governed by `[A-Za-z0-9_]{1,64}`, is tested on its own, and is dropped in
//! **every** form by `FR-ERR-023` where it falls outside, the generic hint then
//! standing alone. The separators that join names — the space between
//! command-path segments, the `-` or `--` that introduces a flag, the `.`
//! between key segments — are literals too, which is why a value carrying one
//! is split before it is tested rather than refused for carrying it.
//!
//! Comparison is over characters as written. `FR-SCH-014` folds ASCII case for
//! the pattern matcher and `FR-ENV-031` for the word-list tokeniser; no
//! requirement extends folding to this path, so a candidate differing only in
//! case is as far away as its number of differing letters.

use std::borrow::Cow;

use super::hint;

/// The widest edit distance `FR-ERR-019` admits a candidate at.
const MAX_DISTANCE: usize = 2;

/// The most suggestions `FR-ERR-019` offers.
const MAX_SUGGESTIONS: usize = 3;

/// The population a candidate was drawn from, which decides how `FR-ERR-022`
/// governs its spelling.
///
/// The eight populations of `FR-ERR-021` fall into four governance classes,
/// and this is that classification rather than the populations themselves. The
/// first three are the spellings this specification enumerates, and are
/// literals: no server, file or caller can influence `cfg database add`,
/// `--ca-file` or `core.render_timeout`. The fourth is every value this corpus
/// does not fix, which the character set governs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Population {
    /// The command tree of `cli-contract.md`: a command, or an alias of one.
    /// A path is written with its segments separated by spaces.
    Commands,
    /// The flags a node declares, written with the `-` or `--` that introduces
    /// them.
    Flags,
    /// The enumerated key space of `FR-CONF-002`. Every segment is enumerated
    /// except the `<name>` of a `database.<name>` key, which is an entry name
    /// and is therefore governed by the character set.
    ConfigurationKeys,
    /// The five populations this corpus does not fix — a table, a view, a
    /// routine, a template, a database entry — together with the name of an
    /// environment variable, which `FR-ERR-022` governs alike.
    Names,
    /// The template names of a project, which `FR-ERR-041` governs: a nested
    /// name carries a `/`, and the set of `FR-ERR-022` would refuse every one.
    Templates,
}

impl Population {
    /// Whether `candidate` may be presented, in a runnable command or as
    /// prose.
    ///
    /// For [`Population::Names`] this is the test `FR-ERR-022` demands and
    /// `FR-ERR-023` acts on: the value comes from outside this corpus and a
    /// spelling outside the set is dropped entirely.
    ///
    /// For the other three the value is a literal, and the test is a
    /// **defensive assertion** rather than a requirement: every command, flag
    /// and key this corpus enumerates passes it, so a candidate that fails came
    /// from somewhere other than the corpus and no runnable command is built
    /// from it. The one exception is the `<name>` segment of a
    /// `database.<name>` key, where the same test is the requirement, exactly
    /// as it is for that entry name suggested in its own right.
    fn admits(self, candidate: &str) -> bool {
        match self {
            Self::Commands => hint::admits_path(candidate),
            Self::Flags => hint::admits_flag(candidate),
            Self::ConfigurationKeys => hint::admits_key(candidate),
            Self::Names => hint::admits(candidate),
            Self::Templates => hint::admits_template(candidate),
        }
    }
}

/// One kept candidate, with the distance that ordered it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Candidate<'a> {
    /// Its distance from the supplied name, which orders it first.
    distance: usize,
    /// The candidate as it is spelled in its population.
    name: &'a str,
}

/// The candidates a selection kept: at most three, in the order `FR-ERR-019`
/// fixes.
///
/// The cap is the array, so "at most three" is structural rather than
/// reviewed, and the selection allocates nothing of its own.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Suggestions<'a> {
    /// The kept candidates, ordered by distance and then by name.
    kept: [Candidate<'a>; MAX_SUGGESTIONS],
    /// How many of [`Suggestions::kept`] are occupied.
    len: usize,
}

impl<'a> Suggestions<'a> {
    /// The kept candidates, in order.
    ///
    /// The iterator is the whole of what a selection exposes: it reports how
    /// many qualified, which is what `FR-ERR-020` turns on, and it is what
    /// [`hint_line`] composes from.
    pub(crate) fn names(&self) -> impl ExactSizeIterator<Item = &'a str> + Clone + '_ {
        self.kept[..self.len].iter().map(|kept| kept.name)
    }

    /// Offers one measured candidate, keeping it where it ranks above a name
    /// already held.
    ///
    /// The order is `FR-ERR-019`'s — distance first, then the name — and a
    /// candidate that ranks below three already kept is discarded without
    /// moving anything.
    fn offer(&mut self, distance: usize, name: &'a str) {
        let position = self.kept[..self.len]
            .iter()
            .position(|kept| (distance, name) < (kept.distance, kept.name))
            .unwrap_or(self.len);

        if position >= MAX_SUGGESTIONS {
            return;
        }

        // Everything from `position` on shifts one place right, and the worst
        // of three falls off the end.
        let end = self.len.min(MAX_SUGGESTIONS - 1);
        self.kept.copy_within(position..end, position + 1);
        self.kept[position] = Candidate { distance, name };
        self.len = (self.len + 1).min(MAX_SUGGESTIONS);
    }
}

/// Selects the nearest matches for `supplied` from `candidates`.
///
/// `population` says how `FR-ERR-022` governs the spelling of a candidate: a
/// candidate it refuses is dropped before it is measured, so it occupies none
/// of the three places and reaches no line of the output, which is
/// `FR-ERR-023`. What survives is measured against `supplied` and kept where it
/// lies within `MAX_DISTANCE`, ordered by distance and then by name.
///
/// **A candidate equal to `supplied` is refused before it is measured**, over
/// every population this function serves. `FR-ERR-019` admits a candidate by
/// its distance and zero is inside the threshold, so nothing in the measure
/// stops one; what stops it is `FR-ERR-038`, which rejects folding ASCII case
/// on the ground that it "would place a name differing only in case at distance
/// **zero** — the measure calling the candidate the supplied name, beneath an
/// `error` line stating that the supplied name does not exist", which
/// `FR-ERR-010` reads as the two lines disagreeing. A suggestion that proposes
/// the very token the invocation was refused for is that disagreement reached
/// by the other road, and the reader is an agent that may act on it.
///
/// The returned names borrow from `candidates`; nothing is copied.
pub(crate) fn suggestions<'a, C>(
    supplied: &str,
    candidates: C,
    population: Population,
) -> Suggestions<'a>
where
    C: IntoIterator<Item = &'a str>,
{
    let mut matrix = Matrix::new(supplied);
    let mut kept = Suggestions::default();

    for candidate in candidates {
        // FR-ERR-038 and FR-ERR-010: a candidate byte-identical to the supplied
        // value is not a suggestion, whatever population it was drawn from.
        if candidate == supplied {
            continue;
        }

        if !population.admits(candidate) {
            continue;
        }

        if let Some(distance) = matrix.distance(candidate, MAX_DISTANCE) {
            kept.offer(distance, candidate);
        }
    }

    kept
}

/// Composes the `hint` line for a name that does not exist.
///
/// The nearest-match half is `FR-ERR-008`'s own example — `did you mean
/// 'orders'? ` — and `generic` follows it, so the line still carries the
/// runnable command `FR-ERR-009` asks for. Where nothing qualified, the generic
/// hint is returned unchanged and unallocated, which is what `FR-ERR-020` and
/// `FR-ERR-023` each require.
///
/// `FR-ERR-008` fixes the wording for one candidate; two are joined by ` or `
/// and three by `, ` and ` or `, so the line reads as one question however many
/// it carries.
///
/// The candidates are taken as an iterator rather than as a [`Suggestions`],
/// because a selection is made where the population lives and the line is
/// composed where the diagnostic is: the two are separated by the error value,
/// which carries the kept names. [`Suggestions::names`] and a slice of names an
/// [`Error`](crate::error::Error) variant carries are therefore both admissible
/// here, and the composition exists once.
pub(crate) fn hint_line<'a, N>(names: N, generic: &'a str) -> Cow<'a, str>
where
    N: ExactSizeIterator + Clone,
    N::Item: AsRef<str>,
{
    let kept = names.len();
    if kept == 0 {
        return Cow::Borrowed(generic);
    }

    // "did you mean " and "? ", plus two quotation marks and a separator of at
    // most four bytes for each name.
    let width = names.clone().map(|name| name.as_ref().len()).sum::<usize>()
        + kept * 6
        + generic.len()
        + 15;
    let mut line = String::with_capacity(width);

    line.push_str("did you mean ");
    let last = kept - 1;
    for (index, name) in names.enumerate() {
        if index > 0 {
            line.push_str(if index == last { " or " } else { ", " });
        }
        line.push('\'');
        line.push_str(name.as_ref());
        line.push('\'');
    }
    line.push_str("? ");
    line.push_str(generic);

    Cow::Owned(line)
}

/// The scratch one run of comparisons shares.
///
/// The supplied name is decoded to characters **once** and the three rolling
/// rows of the distance matrix are allocated **once**, so comparing against a
/// population of 200 names allocates four times rather than eight hundred. The
/// whole of it is `4n` bytes for the characters and `3(n + 1)` words for the
/// rows, over a supplied name of `n` characters: the full matrix is never held,
/// because the recurrence reaches no further back than two rows.
struct Matrix {
    /// The supplied name, decoded once. It indexes the columns.
    supplied: Vec<char>,
    /// Row `i - 2`, which the transposition rule reaches back to.
    two_back: Vec<usize>,
    /// Row `i - 1`.
    one_back: Vec<usize>,
    /// Row `i`, under construction.
    current: Vec<usize>,
}

impl Matrix {
    /// Prepares the scratch for repeated comparison against `supplied`.
    fn new(supplied: &str) -> Self {
        // A character is at least one byte, so the byte length is an exact
        // upper bound on the count and the vector never grows.
        let mut characters = Vec::with_capacity(supplied.len());
        characters.extend(supplied.chars());

        let cells = characters.len() + 1;
        Self {
            supplied: characters,
            two_back: vec![0; cells],
            one_back: vec![0; cells],
            current: vec![0; cells],
        }
    }

    /// The Damerau-Levenshtein distance between the supplied name and
    /// `candidate`, or `None` where it exceeds `ceiling`.
    ///
    /// The measure is `FR-ERR-039`'s: the **restricted** Damerau-Levenshtein
    /// distance, optimal string alignment, in which the insertion, the
    /// deletion and the substitution of one character and the transposition of
    /// two **adjacent** characters each cost one and no substring is edited
    /// twice. A transposition costing one is the property the measure exists
    /// for: `ordres` is one step from `orders`, where plain Levenshtein reports
    /// two.
    ///
    /// The restricted form parts from the unrestricted one only where a further
    /// edit falls between the two transposed characters: `ca` is three steps
    /// from `abc` here and two there, so a candidate of that shape is not
    /// admitted. `FR-ERR-039` rejects the unrestricted form because it reaches
    /// back to an arbitrary earlier row and therefore needs the whole matrix,
    /// which `BR-PERF-004` will not pay for; the three rolling rows below are
    /// the window `OD-20` holds the computation to.
    ///
    /// Two bounds keep the budget of `BR-PERF-004`. A candidate whose length
    /// differs by more than `ceiling` is refused before the matrix is touched,
    /// because each step of an alignment changes the length by at most one; and
    /// a row whose every cell already exceeds `ceiling` ends the computation,
    /// because no later row falls below it.
    fn distance(&mut self, candidate: &str, ceiling: usize) -> Option<usize> {
        let columns = self.supplied.len();
        let rows = candidate.chars().count();

        if rows.abs_diff(columns) > ceiling {
            return None;
        }

        // Row 0: turning the empty prefix of the candidate into the first `j`
        // characters of the supplied name costs one insertion each.
        for (column, cell) in self.one_back.iter_mut().enumerate() {
            *cell = column;
        }

        if rows == 0 {
            // The length bound above has already put this within the ceiling.
            return Some(columns);
        }

        let mut previous: Option<char> = None;
        for (index, character) in candidate.chars().enumerate() {
            let row = index + 1;
            self.current[0] = row;
            let mut row_minimum = row;

            for column in 1..=columns {
                let substitution = usize::from(character != self.supplied[column - 1]);
                let mut cell = (self.one_back[column] + 1)
                    .min(self.current[column - 1] + 1)
                    .min(self.one_back[column - 1] + substitution);

                // Damerau's transposition: the two characters were exchanged,
                // which is one error and not two.
                if column > 1
                    && character == self.supplied[column - 2]
                    && previous == Some(self.supplied[column - 1])
                {
                    cell = cell.min(self.two_back[column - 2] + 1);
                }

                self.current[column] = cell;
                row_minimum = row_minimum.min(cell);
            }

            if row_minimum > ceiling {
                return None;
            }

            previous = Some(character);
            self.advance();
        }

        // The last `advance` moved the final row into `one_back`.
        let measured = self.one_back[columns];
        (measured <= ceiling).then_some(measured)
    }

    /// Advances the window by one row: `i` becomes `i - 1`, and `i - 1`
    /// becomes `i - 2`.
    ///
    /// The row left in [`Matrix::current`] is the one that has just fallen out
    /// of the window, and is overwritten cell by cell before it is read again.
    fn advance(&mut self) {
        std::mem::swap(&mut self.two_back, &mut self.one_back);
        std::mem::swap(&mut self.one_back, &mut self.current);
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_DISTANCE, Matrix, Population, Suggestions, hint_line, suggestions};
    use crate::diagnostics::hint;
    use crate::error::{CatalogueObjectKind, Error};

    /// The generic hint of `FR-ERR-008`'s own example.
    const GENERIC: &str = "list the available tables with: tpl -d shop schema tables";

    /// The four populations, with a candidate each that this corpus admits.
    ///
    /// The list is the classification of [`Population`] and not the eight
    /// populations of `FR-ERR-021`: what a candidate is measured against is the
    /// class, so a property asserted once per class is asserted over every
    /// population the selection serves.
    const EVERY_POPULATION: [(Population, &str); 4] = [
        (Population::Commands, "cfg database add"),
        (Population::Flags, "--pattern"),
        (Population::ConfigurationKeys, "core.render_timeout"),
        (Population::Names, "orders"),
    ];

    #[test]
    fn fr_err_038_no_suggestion_is_ever_the_token_it_is_offered_for() {
        // FR-ERR-019 admits a candidate at distance zero, and a candidate equal
        // to the supplied value is at zero — which is the measure calling the
        // candidate the supplied name, beneath an `error` line saying the
        // supplied name does not exist. FR-ERR-038 rejects folding case on
        // exactly that ground and FR-ERR-010 reads the two lines as one thing,
        // so the candidate is dropped before it is measured, over every
        // population.
        for (population, candidate) in EVERY_POPULATION {
            assert_eq!(
                kept(candidate, &[candidate], population),
                Vec::<&str>::new(),
                "{population:?} offered the token it was given"
            );

            // The drop is of that candidate and of nothing else: a neighbour of
            // the same population is still offered beside it.
            let neighbour = match population {
                Population::Commands => "cfg database ad",
                Population::Flags => "--patern",
                Population::ConfigurationKeys => "core.render_timeut",
                Population::Names => "order",
                Population::Templates => "rust/struct",
            };

            assert_eq!(
                kept(candidate, &[candidate, neighbour], population),
                vec![neighbour],
                "{population:?} dropped more than the token it was given"
            );
        }
    }

    /// The measured distance between two names, with no ceiling in the way.
    fn measured(supplied: &str, candidate: &str) -> Option<usize> {
        Matrix::new(supplied).distance(candidate, MAX_DISTANCE)
    }

    /// The names a selection kept, in order.
    fn kept<'a>(supplied: &str, candidates: &[&'a str], population: Population) -> Vec<&'a str> {
        suggestions(supplied, candidates.iter().copied(), population)
            .names()
            .collect()
    }

    /// The distance, computed over the whole matrix and with no bound of any
    /// kind.
    ///
    /// [`Matrix`] holds three rows rather than the matrix, refuses a candidate
    /// on its length, and abandons a row whose every cell exceeds the ceiling.
    /// This is the same recurrence written the obvious way, so that the three
    /// economies can be checked against it rather than reasoned about.
    fn reference(candidate: &[char], supplied: &[char]) -> usize {
        let mut matrix = vec![vec![0usize; supplied.len() + 1]; candidate.len() + 1];

        for (row, cells) in matrix.iter_mut().enumerate() {
            cells[0] = row;
        }
        for (column, cell) in matrix[0].iter_mut().enumerate() {
            *cell = column;
        }

        for row in 1..=candidate.len() {
            for column in 1..=supplied.len() {
                let substitution = usize::from(candidate[row - 1] != supplied[column - 1]);
                let mut cell = (matrix[row - 1][column] + 1)
                    .min(matrix[row][column - 1] + 1)
                    .min(matrix[row - 1][column - 1] + substitution);

                if row > 1
                    && column > 1
                    && candidate[row - 1] == supplied[column - 2]
                    && candidate[row - 2] == supplied[column - 1]
                {
                    cell = cell.min(matrix[row - 2][column - 2] + 1);
                }

                matrix[row][column] = cell;
            }
        }

        matrix[candidate.len()][supplied.len()]
    }

    /// Every string of up to four characters over a three-letter alphabet.
    fn corpus() -> Vec<String> {
        let mut all = vec![String::new()];
        let mut frontier = vec![String::new()];

        for _ in 0..4 {
            let mut grown = Vec::with_capacity(frontier.len() * 3);
            for word in &frontier {
                for letter in ['a', 'b', 'c'] {
                    let mut longer = word.clone();
                    longer.push(letter);
                    grown.push(longer);
                }
            }
            all.extend(grown.iter().cloned());
            frontier = grown;
        }

        all
    }

    // ----------------------------------------------------- the distance ---

    #[test]
    fn fr_err_019_a_transposition_is_one_error_and_not_two() {
        // OD-20's worked example, which is observable output: plain
        // Levenshtein reports 2 for this pair and the candidate set differs on
        // it, because FR-ERR-019 admits a candidate by its distance.
        assert_eq!(measured("ordres", "orders"), Some(1));

        // The same pair, measured the other way: the distance is symmetric.
        assert_eq!(measured("orders", "ordres"), Some(1));
    }

    #[test]
    fn fr_err_019_a_transposition_split_by_a_further_edit_costs_three() {
        // The restricted form's one departure from the unrestricted one, held
        // deliberately: 'ca' reaches 'abc' in two steps only by transposing and
        // then inserting between the transposed pair, which is the substring
        // edited twice that the restricted form refuses. The unrestricted form
        // would need the whole matrix on a budgeted path.
        assert_eq!(measured("ca", "abc"), None);

        // The transposition itself is one error either way.
        assert_eq!(measured("ca", "ac"), Some(1));
    }

    #[test]
    fn fr_err_019_two_substitutions_are_admitted_and_three_are_not() {
        // FR-ERR-019 admits "within an edit distance of two".
        assert_eq!(measured("orders", "ordert"), Some(1));
        assert_eq!(measured("orders", "ordezz"), Some(2));
        assert_eq!(measured("orders", "ordzzz"), None);
    }

    #[test]
    fn fr_err_019_the_three_elementary_edits_each_cost_one() {
        assert_eq!(measured("orders", "order"), Some(1), "a deletion");
        assert_eq!(measured("orders", "borders"), Some(1), "an insertion");
        assert_eq!(measured("orders", "orderz"), Some(1), "a substitution");
        assert_eq!(measured("orders", "orders"), Some(0), "no edit at all");
    }

    #[test]
    fn br_perf_004_a_name_of_a_different_length_is_refused_before_it_is_measured() {
        // The length bound, which is what keeps BR-PERF-004's 200 comparisons
        // cheap: three characters of difference cannot be two edits.
        assert_eq!(measured("orders", "ord"), None);
        assert_eq!(measured("orders", "ordersabc"), None);
    }

    #[test]
    fn fr_err_019_an_empty_side_is_measured_by_the_length_of_the_other() {
        assert_eq!(measured("", "ab"), Some(2));
        assert_eq!(measured("ab", ""), Some(2));
        assert_eq!(measured("", ""), Some(0));
        assert_eq!(measured("", "abc"), None);
    }

    #[test]
    fn fr_err_019_the_three_economies_agree_with_the_whole_matrix_on_every_short_pair() {
        // 121 strings over {a, b, c}, so 14 641 ordered pairs: the rolling
        // rows, the length refusal and the abandoned row are each checked
        // against the recurrence written out in full.
        let corpus = corpus();
        let decoded: Vec<Vec<char>> = corpus.iter().map(|word| word.chars().collect()).collect();

        for (supplied, supplied_characters) in corpus.iter().zip(&decoded) {
            let mut matrix = Matrix::new(supplied);

            for (candidate, candidate_characters) in corpus.iter().zip(&decoded) {
                let expected = reference(candidate_characters, supplied_characters);
                let measured = matrix.distance(candidate, MAX_DISTANCE);

                assert_eq!(
                    measured,
                    (expected <= MAX_DISTANCE).then_some(expected),
                    "{supplied:?} against {candidate:?}"
                );
            }
        }
    }

    #[test]
    fn fr_err_019_the_distance_counts_characters_and_not_bytes() {
        // A supplied name is arbitrary UTF-8; one character replaced is one
        // edit however many bytes it occupies.
        assert_eq!(
            measured("encomendas_pag\u{e1}s", "encomendas_pagas"),
            Some(1)
        );
    }

    // ------------------------------------------- the selection and its cap ---

    #[test]
    fn fr_err_019_four_candidates_within_two_yield_three_by_distance_then_name() {
        // 'aorders' and 'orderz' are one edit away, 'border' and 'xorderz' two.
        let population = ["xorderz", "border", "orderz", "aorders"];

        assert_eq!(
            kept("orders", &population, Population::Names),
            ["aorders", "orderz", "border"],
            "distance orders first and the name breaks the tie"
        );
    }

    #[test]
    fn fr_err_020_nothing_within_the_distance_yields_nothing() {
        // FR-ERR-020: the suggestion is omitted rather than weakened.
        let selected = suggestions(
            "orders",
            ["customers", "invoices", "shipments"],
            Population::Names,
        );

        assert_eq!(selected.names().len(), 0);
    }

    #[test]
    fn fr_err_022_a_name_of_sixty_four_characters_is_admitted_and_one_of_sixty_five_is_not() {
        let supplied = format!("{}b", "a".repeat(63));
        let admitted = "a".repeat(64);
        let refused = "a".repeat(65);

        assert_eq!(
            kept(
                &supplied,
                &[refused.as_str(), admitted.as_str()],
                Population::Names
            ),
            [admitted.as_str()],
            "{{1,64}} is the bound FR-ERR-022 states"
        );
    }

    // --------------------------------------- FR-ERR-022 and FR-ERR-023 ---

    #[test]
    fn fr_err_023_a_candidate_outside_the_set_is_presented_in_no_form() {
        // FR-ERR-023: the candidate is near enough to be suggested and is
        // dropped anyway, because a table name is free text on the server.
        let hostile = "orders;DROP TABLE x";
        let selected = suggestions("orders;DROP TABLE y", [hostile], Population::Names);

        assert_eq!(selected.names().len(), 0, "the candidate must not be kept");

        let line = hint_line(selected.names(), GENERIC);
        assert_eq!(line, GENERIC, "the generic hint stands alone");
        assert!(!line.contains("DROP"), "{line}");
        assert!(!line.contains(';'), "{line}");
        assert!(!line.contains("did you mean"), "{line}");
    }

    #[test]
    fn fr_err_022_a_key_of_fr_conf_002_is_suggested_and_is_not_dropped_for_its_dot() {
        // FR-ERR-022 as amended in the twentieth edition: a key is a spelling
        // this specification enumerates, and no key matches the character set
        // as a whole because all eighteen forms carry a dot.
        let population = [
            "core.database",
            "core.connect_timeout",
            "core.query_timeout",
            "core.render_timeout",
        ];

        assert_eq!(
            kept("core.databse", &population, Population::ConfigurationKeys),
            ["core.database"]
        );
    }

    #[test]
    fn fr_err_022_a_hyphenated_flag_is_suggested_and_is_not_dropped_for_its_hyphen() {
        // The five flags of this corpus that carry a hyphen inside the name.
        let population = [
            "--ca-file",
            "--ca-path",
            "--no-cache",
            "--tpl-dir",
            "--password-command",
        ];

        assert_eq!(
            kept("--ca-fil", &population, Population::Flags),
            ["--ca-file"]
        );
        assert!(Population::Flags.admits("-d"), "a short flag is a flag");
    }

    #[test]
    fn fr_err_022_a_database_key_is_tested_segment_by_segment() {
        // FR-ERR-022's consequence: the key is admissible exactly as far as
        // the entry name inside it is.
        assert!(Population::ConfigurationKeys.admits("database.reporting.host"));
        assert!(!Population::ConfigurationKeys.admits("database.report;drop.host"));

        // And the same entry name, suggested in its own right, is dropped by
        // the same test.
        assert!(!Population::Names.admits("report;drop"));
    }

    #[test]
    fn fr_err_023_a_database_key_naming_an_entry_outside_the_set_reaches_no_line() {
        let hostile = "database.rm -rf /.host";
        let selected = suggestions(
            "database.rm -rf /.hosts",
            [hostile, "database.reporting.host"],
            Population::ConfigurationKeys,
        );

        assert_eq!(selected.names().len(), 0, "neither key qualifies");
        assert!(!hint_line(selected.names(), GENERIC).contains("rm -rf"));
    }

    #[test]
    fn fr_err_022_a_command_path_is_admitted_over_its_segments() {
        assert!(Population::Commands.admits("cfg database add"));
        assert!(!Population::Commands.admits("cfg database add; rm"));
    }

    #[test]
    fn fr_err_022_an_empty_or_malformed_spelling_is_refused_by_every_population() {
        for population in [
            Population::Commands,
            Population::Flags,
            Population::ConfigurationKeys,
            Population::Names,
        ] {
            assert!(
                !population.admits(""),
                "{population:?} admitted an empty name"
            );
        }

        assert!(!Population::Flags.admits("--"), "a flag with no name");
        assert!(!Population::Flags.admits("--ca-"), "an empty flag segment");
        assert!(
            !Population::ConfigurationKeys.admits("core."),
            "an empty key segment"
        );
    }

    // -------------------------------------------------- the composed line ---

    #[test]
    fn fr_err_008_one_candidate_composes_the_line_the_requirement_shows() {
        let error = Error::CatalogueObjectNotFound {
            kind: CatalogueObjectKind::Table,
            name: "ordrs".to_owned(),
            entry: "shop".to_owned(),
            database: "shop".to_owned(),
            nearest: Vec::new(),
        };
        let selected = suggestions("ordrs", ["orders", "customers"], Population::Names);

        assert_eq!(
            hint_line(selected.names(), &hint::hint(&error)),
            "did you mean 'orders'? list the available tables with: tpl -d shop schema tables"
        );
    }

    #[test]
    fn fr_err_008_two_and_three_candidates_each_read_as_one_question() {
        let two = suggestions("orders", ["orderz", "aorders"], Population::Names);
        assert_eq!(
            hint_line(two.names(), GENERIC),
            format!("did you mean 'aorders' or 'orderz'? {GENERIC}")
        );

        let three = suggestions("orders", ["orderz", "aorders", "border"], Population::Names);
        assert_eq!(
            hint_line(three.names(), GENERIC),
            format!("did you mean 'aorders', 'orderz' or 'border'? {GENERIC}")
        );
    }

    #[test]
    fn an_empty_selection_borrows_the_generic_hint_rather_than_copying_it() {
        let line = hint_line(Suggestions::default().names(), GENERIC);

        assert_eq!(line, GENERIC);
        assert!(matches!(line, std::borrow::Cow::Borrowed(_)));
    }
}
