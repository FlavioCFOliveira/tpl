//! A column of a table or a view (`FR-SCH-009`, `FR-CTX-019` … `FR-CTX-021`).
//!
//! The column is where the model refuses most visibly. `FR-CTX-021` forbids
//! materialising on a column any fact its table already states, and names two:
//! `is_primary_key` and `is_unique`. Neither is a field here, and the refusal
//! is structural — `BR-CTX-003` is the reason, and it is a reason about
//! documents rather than about bytes. A column that said it was not part of the
//! primary key while its table said it was would be a document that contradicts
//! itself, and nothing downstream could repair it once both were written.
//! `FR-CTX-022` is where those two facts are answered instead: the tests
//! `primary_key` and `unique` resolve [`Column::table_name`] against the render
//! context and ask the table.
//!
//! | Requirement | What the column does with it |
//! |---|---|
//! | `FR-SCH-009` — position, type, nullability, default, comment, generated status | One field each, and the type and the default are the decompositions of [`super::column_type`] and [`super::column_default`] |
//! | `FR-CTX-019` — every column names its table | [`Column::table_name`], which is what makes a column reached through an index or a key self-locating |
//! | `FR-CTX-020`, `FR-CAT-041` — the static attributes the catalogue states | Four fields, read from the one attribute field `FR-CAT-041` fixes |
//! | `FR-CAT-027` — the auto-increment attribute stays | [`Column::auto_increment`], which is **not** the table counter `FR-CAT-024` excludes |
//! | `FR-CAT-051` — a generated column carries its expression and storage kind | [`Generated`], one value carrying both, so neither can be present without the other |

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::column_default::ColumnDefault;
use super::column_type::ColumnType;

/// The attribute-field value of a column generated and not stored.
const VIRTUAL_GENERATED: &str = "VIRTUAL GENERATED";

/// The attribute-field value of a column generated and stored.
const STORED_GENERATED: &str = "STORED GENERATED";

/// Whether a generated column's value is stored or recomputed (`FR-CAT-051`).
///
/// `FR-CAT-051` reads the storage kind from the column attribute field and
/// forbids reading it from the is-generated field, which says only *whether* a
/// column is generated and never *how*. The two values above are the two that
/// field was observed to take for a generated column, on all four series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GeneratedStorage {
    /// The value is recomputed on every read.
    #[serde(rename = "VIRTUAL GENERATED")]
    Virtual,

    /// The value is written to the table.
    #[serde(rename = "STORED GENERATED")]
    Stored,
}

impl GeneratedStorage {
    /// Reads the storage kind from the column attribute field of
    /// `FR-CAT-041`.
    ///
    /// [`None`] for every other value the field takes, which is the column
    /// that is not generated at all.
    #[must_use]
    pub fn from_attribute(attribute: &str) -> Option<Self> {
        match attribute {
            VIRTUAL_GENERATED => Some(Self::Virtual),
            STORED_GENERATED => Some(Self::Stored),
            _ => None,
        }
    }

    /// The attribute-field value this kind was read from.
    #[must_use]
    pub const fn attribute(self) -> &'static str {
        match self {
            Self::Virtual => VIRTUAL_GENERATED,
            Self::Stored => STORED_GENERATED,
        }
    }
}

/// What a generated column adds to an ordinary one (`FR-CAT-051`).
///
/// The expression and the storage kind are one value because a column has
/// either both or neither: an expression without a storage kind would not say
/// whether the value is written, and a storage kind without an expression would
/// describe nothing. [`Column::generated`] is therefore an [`Option`] of this
/// rather than two independent optional fields, and the half-populated state
/// has no representation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Generated<'a> {
    /// The generation expression, as the catalogue rewrote it: identifiers
    /// backtick-quoted, function names lower-cased, carrying no `AS`, no
    /// enclosing parentheses, and neither the `VIRTUAL` nor the `STORED`
    /// keyword.
    #[serde(borrow)]
    pub expression: Cow<'a, str>,

    /// Whether the value is stored, read from the attribute field and never
    /// from the is-generated field.
    pub storage: GeneratedStorage,
}

/// One column of a table or a view.
///
/// The fields are public and the type is `#[non_exhaustive]`, so the model can
/// be read anywhere and built only inside this crate. There is no invariant for
/// a constructor to enforce: every field is a fact read from one catalogue
/// field, and the two that are decompositions rather than readings —
/// [`Column::column_type`] and [`Column::default`] — enforce their own.
///
/// [`Deserialize`] is written by hand, in [`decode`], and reads exactly the
/// flat object the derived [`Serialize`] writes; the reason is given there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Column<'a> {
    /// The column's name, returned unescaped and unquoted. Quoting it for a
    /// target dialect is the template's job, per `FR-ENV-045`.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// The table the column belongs to (`FR-CTX-019`).
    #[serde(borrow)]
    pub table_name: Cow<'a, str>,

    /// The ordinal position, one-based, which is also the order `FR-CAT-009`
    /// presents the columns in.
    pub position: u64,

    /// The type: the raw string of `FR-CTX-014` and the eight decomposed parts
    /// of `FR-CTX-015`, carried as **siblings** on the column.
    ///
    /// `FR-CTX-014` gives a column `column_type`; `FR-CTX-015` gives it the
    /// decomposed parts *additionally*. Both read as fields of the column, so
    /// a serialised column reads `col.data_type` and never
    /// `col.column_type.data_type`, and `#[serde(flatten)]` is what makes the
    /// document say so. The decomposition remains a type of its own, because
    /// [`ColumnType::decompose`](super::column_type::ColumnType::decompose)
    /// being its only constructor is what makes `FR-CTX-040` structural — the
    /// nesting was an artefact of that type, not a rule of the document.
    ///
    /// The nine keys are emitted where this field sits, so `OD-18` still holds
    /// and the key order is the field order of this struct with
    /// [`ColumnType`]'s own order spliced in at this position.
    ///
    /// The attribute governs the serialisation only: the decoding of the same
    /// flat object is [`decode`]'s, which reads the nine keys without the
    /// buffering the attribute imposes on a derived decoding.
    #[serde(flatten)]
    pub column_type: ColumnType<'a>,

    /// Whether the column admits `NULL`, which is what the `nullable` test of
    /// `FR-ENV-041` answers from.
    pub nullable: bool,

    /// The default, classified per `FR-CTX-037`. [`None`] is the one case with
    /// no default at all: a `NOT NULL` column that declares none.
    pub default: Option<ColumnDefault<'a>>,

    /// The comment, which is the **empty string** where none was given and is
    /// never absent, per `FR-CAT-039`. It carries no promise of being the text
    /// the author wrote, per `FR-CAT-036`.
    #[serde(borrow)]
    pub comment: Cow<'a, str>,

    /// Whether the column is auto-incremental (`FR-CAT-027`, `FR-CTX-020`).
    ///
    /// This is the **column attribute**. The table-level counter of the same
    /// name is excluded outright by `FR-CAT-024` and has no field anywhere.
    pub auto_increment: bool,

    /// Whether the column is invisible (`FR-CAT-035`, `FR-CTX-020`). An
    /// invisible column is carried like any other, per `FR-CAT-009`.
    pub invisible: bool,

    /// The generation expression and storage kind, or [`None`] for a column
    /// that is not generated (`FR-CAT-051`).
    pub generated: Option<Generated<'a>>,

    /// The `ON UPDATE` default the attribute field states, where it states one
    /// (`FR-CAT-041`, `FR-CTX-020`).
    ///
    /// It is carried verbatim — `on update current_timestamp()` and
    /// `on update current_timestamp(3)` were both observed — because it is
    /// reported nowhere else: the column-default field of `FR-CTX-037` carries
    /// the `DEFAULT` clause and says nothing about `ON UPDATE`.
    #[serde(borrow)]
    pub on_update: Option<Cow<'a, str>>,
}

/// The hand-written [`Deserialize`] of [`Column`] (`FR-SCH-022`,
/// `FR-CACHE-033`, `FR-RND-020`).
///
/// # Why it is not derived
///
/// `#[serde(flatten)]` on [`Column::column_type`] makes a derived decoding
/// buffer every key it does not name itself into serde's private `Content`
/// tree, and then decode the type from that tree once the object has ended.
/// The buffering allocates a node per value, copies every string it could have
/// borrowed, and was 13–19% of the samples of every cached read of a whole
/// collection: 12 094 208 B in 20 247 blocks for the 2 400 columns of `WL-001`
/// (`BENCHMARKS.md`, 2026-09-22). The shape of the document is untouched by how
/// it is read; the buffering was an artefact of the derive.
///
/// # What it preserves, and how
///
/// The decoding accepts the same documents as the derive did, into equal
/// values, and refuses every other document in the same `serde_json` category
/// at the same position — which is everything a caller observes: the cache
/// makes every refusal a miss, and `--context` reports the category, with the
/// position where the text is not JSON. The tests of this file decide a corpus
/// of more than a thousand documents against a copy of the derive.
///
/// - **The column's own ten keys** are decoded as they are met, into the same
///   types, as the derive decoded them: a duplicate is refused at once, and a
///   missing key is refused where the field is required and read as absent
///   where it is an [`Option`].
/// - **The nine keys of [`ColumnType::MEMBERS`]** are parsed as they are met
///   into [`Seen`], a value that holds what the text said and borrows every
///   string that needed no unescaping, and are decoded by [`ColumnType`]'s own
///   derive once the object has ended — which is when the derive decoded them
///   from its buffer. A value of the wrong kind is therefore refused after the
///   rest of the object has been read, and text that is not JSON further on is
///   what a decode reports, exactly as before.
/// - **Any other key** is parsed in full by [`Strict`] and dropped. It is
///   parsed rather than skipped because the derive parsed it into its buffer:
///   a number out of range or a lone surrogate under an unknown key made the
///   document not JSON, and still does.
/// - **Anything but an object** is refused: a flattened derive decodes a map
///   and nothing else.
///
/// *Rejected: a derived decoding of a private mirror of the flat shape.* Its
/// nine type members would be decoded as they are met, which moves a refusal
/// of the wrong kind of value ahead of text that is not JSON further on, and
/// [`ColumnType`] would need a second constructor beside
/// [`ColumnType::decompose`].
mod decode {
    use std::borrow::Cow;
    use std::fmt;
    use std::marker::PhantomData;

    use serde::Deserialize;
    use serde::de::value::{SeqDeserializer, StrDeserializer};
    use serde::de::{
        self, Deserializer, IntoDeserializer, MapAccess, SeqAccess, Unexpected, Visitor,
    };

    use super::{Column, ColumnDefault, ColumnType, Generated};

    /// The name the refusals of a missing or duplicated key are reported
    /// under.
    const NAME: &str = "name";
    /// See [`NAME`].
    const TABLE_NAME: &str = "table_name";
    /// See [`NAME`].
    const POSITION: &str = "position";
    /// See [`NAME`].
    const NULLABLE: &str = "nullable";
    /// See [`NAME`].
    const DEFAULT: &str = "default";
    /// See [`NAME`].
    const COMMENT: &str = "comment";
    /// See [`NAME`].
    const AUTO_INCREMENT: &str = "auto_increment";
    /// See [`NAME`].
    const INVISIBLE: &str = "invisible";
    /// See [`NAME`].
    const GENERATED: &str = "generated";
    /// See [`NAME`].
    const ON_UPDATE: &str = "on_update";

    impl<'de: 'a, 'a> Deserialize<'de> for Column<'a> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            // A flattened derive asks for a map, so a sequence is refused as
            // it was.
            deserializer.deserialize_map(Columns(PhantomData))
        }
    }

    /// Which field one key of a column object names.
    enum Key {
        /// One of the column's own ten keys.
        Own(Own),
        /// The key at this index of [`ColumnType::MEMBERS`].
        Type(usize),
        /// A key no field names, which is parsed and dropped.
        Other,
    }

    /// The column's own ten keys.
    #[derive(Clone, Copy)]
    enum Own {
        /// `name`.
        Name,
        /// `table_name`.
        TableName,
        /// `position`.
        Position,
        /// `nullable`.
        Nullable,
        /// `default`.
        Default,
        /// `comment`.
        Comment,
        /// `auto_increment`.
        AutoIncrement,
        /// `invisible`.
        Invisible,
        /// `generated`.
        Generated,
        /// `on_update`.
        OnUpdate,
    }

    impl Key {
        /// The field `key` names.
        fn of(key: &str) -> Self {
            match key {
                NAME => Self::Own(Own::Name),
                TABLE_NAME => Self::Own(Own::TableName),
                POSITION => Self::Own(Own::Position),
                NULLABLE => Self::Own(Own::Nullable),
                DEFAULT => Self::Own(Own::Default),
                COMMENT => Self::Own(Own::Comment),
                AUTO_INCREMENT => Self::Own(Own::AutoIncrement),
                INVISIBLE => Self::Own(Own::Invisible),
                GENERATED => Self::Own(Own::Generated),
                ON_UPDATE => Self::Own(Own::OnUpdate),
                other => ColumnType::MEMBERS
                    .iter()
                    .position(|member| *member == other)
                    .map_or(Self::Other, Self::Type),
            }
        }
    }

    impl<'de> Deserialize<'de> for Key {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_identifier(Keys)
        }
    }

    /// The visitor of [`Key`].
    struct Keys;

    impl Visitor<'_> for Keys {
        type Value = Key;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a field identifier")
        }

        fn visit_str<E: de::Error>(self, key: &str) -> Result<Key, E> {
            Ok(Key::of(key))
        }

        fn visit_bytes<E: de::Error>(self, key: &[u8]) -> Result<Key, E> {
            Ok(std::str::from_utf8(key).map_or(Key::Other, Key::of))
        }
    }

    /// A string field decoded as the derive decodes a `#[serde(borrow)]`
    /// [`Cow`]: borrowed where the text needed no unescaping, owned where it
    /// did.
    #[derive(Deserialize)]
    struct Text<'a>(#[serde(borrow)] Cow<'a, str>);

    /// The visitor of [`Column`].
    struct Columns<'a>(PhantomData<Column<'a>>);

    impl<'de: 'a, 'a> Visitor<'de> for Columns<'a> {
        type Value = Column<'a>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("struct Column")
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Column<'a>, A::Error> {
            let mut name: Option<Cow<'a, str>> = None;
            let mut table_name: Option<Cow<'a, str>> = None;
            let mut position: Option<u64> = None;
            let mut nullable: Option<bool> = None;
            let mut default: Option<Option<ColumnDefault<'a>>> = None;
            let mut comment: Option<Cow<'a, str>> = None;
            let mut auto_increment: Option<bool> = None;
            let mut invisible: Option<bool> = None;
            let mut generated: Option<Option<Generated<'a>>> = None;
            let mut on_update: Option<Option<Cow<'a, str>>> = None;
            let mut typed = Typed::default();

            while let Some(key) = map.next_key::<Key>()? {
                match key {
                    Key::Own(own) => match own {
                        Own::Name => once(&mut name, NAME, || {
                            map.next_value::<Text<'de>>().map(|text| text.0)
                        })?,
                        Own::TableName => once(&mut table_name, TABLE_NAME, || {
                            map.next_value::<Text<'de>>().map(|text| text.0)
                        })?,
                        Own::Position => once(&mut position, POSITION, || map.next_value())?,
                        Own::Nullable => once(&mut nullable, NULLABLE, || map.next_value())?,
                        Own::Default => once(&mut default, DEFAULT, || map.next_value())?,
                        Own::Comment => once(&mut comment, COMMENT, || {
                            map.next_value::<Text<'de>>().map(|text| text.0)
                        })?,
                        Own::AutoIncrement => {
                            once(&mut auto_increment, AUTO_INCREMENT, || map.next_value())?;
                        }
                        Own::Invisible => once(&mut invisible, INVISIBLE, || map.next_value())?,
                        Own::Generated => once(&mut generated, GENERATED, || map.next_value())?,
                        Own::OnUpdate => once(&mut on_update, ON_UPDATE, || map.next_value())?,
                    },
                    Key::Type(at) => typed.put(at, map.next_value::<Seen<'de>>()?),
                    Key::Other => {
                        map.next_value::<Strict>()?;
                    }
                }
            }

            // The order the derive checked in: the fields in declaration
            // order, the flattened type in its place among them.
            Ok(Column {
                name: required(name, NAME)?,
                table_name: required(table_name, TABLE_NAME)?,
                position: required(position, POSITION)?,
                column_type: typed.decode()?,
                nullable: required(nullable, NULLABLE)?,
                default: default.flatten(),
                comment: required(comment, COMMENT)?,
                auto_increment: required(auto_increment, AUTO_INCREMENT)?,
                invisible: required(invisible, INVISIBLE)?,
                generated: generated.flatten(),
                on_update: on_update.flatten(),
            })
        }
    }

    /// Decodes one of the column's own keys into `slot`, refusing a second
    /// occurrence before its value is read, as the derive did.
    fn once<T, E: de::Error>(
        slot: &mut Option<T>,
        key: &'static str,
        read: impl FnOnce() -> Result<T, E>,
    ) -> Result<(), E> {
        if slot.is_some() {
            return Err(E::duplicate_field(key));
        }
        *slot = Some(read()?);
        Ok(())
    }

    /// The value of a required key, or the refusal of its absence.
    fn required<T, E: de::Error>(slot: Option<T>, key: &'static str) -> Result<T, E> {
        slot.ok_or_else(|| E::missing_field(key))
    }

    /// The nine type keys met so far, each as the text stated it.
    #[derive(Default)]
    struct Typed<'de> {
        /// One slot per key of [`ColumnType::MEMBERS`], in that order.
        slots: [Option<Seen<'de>>; 9],
        /// The first key met twice, which the decode refuses.
        twice: Option<&'static str>,
    }

    impl<'de> Typed<'de> {
        /// Records the value of the key at `at`.
        fn put(&mut self, at: usize, seen: Seen<'de>) {
            if let Some(slot) = self.slots.get_mut(at) {
                if slot.is_some() {
                    self.twice = self.twice.or(ColumnType::MEMBERS.get(at).copied());
                }
                *slot = Some(seen);
            }
        }

        /// The type the recorded keys describe, decoded by [`ColumnType`]'s
        /// own derive.
        fn decode<'a, E: de::Error>(self) -> Result<ColumnType<'a>, E>
        where
            'de: 'a,
        {
            if let Some(key) = self.twice {
                return Err(E::duplicate_field(key));
            }

            ColumnType::deserialize(de::value::MapAccessDeserializer::new(Recorded {
                entries: ColumnType::MEMBERS.into_iter().zip(self.slots),
                pending: None,
                error: PhantomData,
            }))
        }
    }

    /// The recorded type keys, handed to [`ColumnType`]'s derive as a map.
    struct Recorded<'de, I, E> {
        /// The keys, each with its value where the object carried it.
        entries: I,
        /// The value of the key last handed out.
        pending: Option<Seen<'de>>,
        /// The error type of the decoder the column is read from.
        error: PhantomData<E>,
    }

    impl<'de, I, E> MapAccess<'de> for Recorded<'de, I, E>
    where
        I: Iterator<Item = (&'static str, Option<Seen<'de>>)>,
        E: de::Error,
    {
        type Error = E;

        fn next_key_seed<K: de::DeserializeSeed<'de>>(
            &mut self,
            seed: K,
        ) -> Result<Option<K::Value>, E> {
            for (key, value) in self.entries.by_ref() {
                if let Some(value) = value {
                    self.pending = Some(value);
                    return seed.deserialize(StrDeserializer::<E>::new(key)).map(Some);
                }
            }
            Ok(None)
        }

        fn next_value_seed<V: de::DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, E> {
            let value = self
                .pending
                .take()
                .ok_or_else(|| E::custom("a value was asked for before its key"))?;
            seed.deserialize(value.into_deserializer())
        }
    }

    /// One JSON value, held as the text stated it (`Content`'s counterpart,
    /// for the values a column type admits).
    enum Seen<'de> {
        /// `null`.
        Null,
        /// `true` or `false`.
        Bool(bool),
        /// A non-negative integer.
        Unsigned(u64),
        /// A negative integer.
        Signed(i64),
        /// A number with a fraction or an exponent.
        Float(f64),
        /// A string, borrowed where it needed no unescaping.
        Text(Cow<'de, str>),
        /// An array.
        Array(Vec<Seen<'de>>),
        /// An object, parsed in full; no column-type value is one.
        Object,
    }

    impl<'de> Deserialize<'de> for Seen<'de> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_any(Seeing)
        }
    }

    /// The visitor of [`Seen`].
    struct Seeing;

    impl<'de> Visitor<'de> for Seeing {
        type Value = Seen<'de>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("any value")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Seen<'de>, E> {
            Ok(Seen::Null)
        }

        fn visit_none<E: de::Error>(self) -> Result<Seen<'de>, E> {
            Ok(Seen::Null)
        }

        fn visit_bool<E: de::Error>(self, value: bool) -> Result<Seen<'de>, E> {
            Ok(Seen::Bool(value))
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<Seen<'de>, E> {
            Ok(Seen::Unsigned(value))
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<Seen<'de>, E> {
            Ok(Seen::Signed(value))
        }

        fn visit_f64<E: de::Error>(self, value: f64) -> Result<Seen<'de>, E> {
            Ok(Seen::Float(value))
        }

        fn visit_borrowed_str<E: de::Error>(self, value: &'de str) -> Result<Seen<'de>, E> {
            Ok(Seen::Text(Cow::Borrowed(value)))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Seen<'de>, E> {
            Ok(Seen::Text(Cow::Owned(value.to_owned())))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Seen<'de>, E> {
            Ok(Seen::Text(Cow::Owned(value)))
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Seen<'de>, A::Error> {
            let mut members = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(member) = seq.next_element()? {
                members.push(member);
            }
            Ok(Seen::Array(members))
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Seen<'de>, A::Error> {
            while map.next_entry::<Strict, Strict>()?.is_some() {}
            Ok(Seen::Object)
        }
    }

    /// The deserializer over one [`Seen`] value, as serde's own
    /// `ContentDeserializer` answers over a buffered value.
    struct Replay<'de, E> {
        /// The value replayed.
        seen: Seen<'de>,
        /// The error type of the decoder the column is read from.
        error: PhantomData<E>,
    }

    impl<'de, E: de::Error> IntoDeserializer<'de, E> for Seen<'de> {
        type Deserializer = Replay<'de, E>;

        fn into_deserializer(self) -> Replay<'de, E> {
            Replay {
                seen: self,
                error: PhantomData,
            }
        }
    }

    impl<'de, E: de::Error> Deserializer<'de> for Replay<'de, E> {
        type Error = E;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
            match self.seen {
                Seen::Null => visitor.visit_unit(),
                Seen::Bool(value) => visitor.visit_bool(value),
                Seen::Unsigned(value) => visitor.visit_u64(value),
                Seen::Signed(value) => visitor.visit_i64(value),
                Seen::Float(value) => visitor.visit_f64(value),
                Seen::Text(Cow::Borrowed(value)) => visitor.visit_borrowed_str(value),
                Seen::Text(Cow::Owned(value)) => visitor.visit_string(value),
                Seen::Array(members) => {
                    let mut seq = SeqDeserializer::new(members.into_iter());
                    let value = visitor.visit_seq(&mut seq)?;
                    seq.end()?;
                    Ok(value)
                }
                Seen::Object => Err(E::invalid_type(Unexpected::Map, &visitor)),
            }
        }

        fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
            match self.seen {
                Seen::Null => visitor.visit_none(),
                _ => visitor.visit_some(self),
            }
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    /// Any JSON value, parsed in full and dropped.
    ///
    /// It is serde's `IgnoredAny` with one difference, and the difference is
    /// the reason for it: a skipped number is not converted and a skipped
    /// string is not unescaped, so a value out of range or a lone surrogate
    /// would pass, where the derive's buffer refused both.
    struct Strict;

    impl<'de> Deserialize<'de> for Strict {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_any(Strict)
        }
    }

    impl<'de> Visitor<'de> for Strict {
        type Value = Self;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("any value")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_none<E: de::Error>(self) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_str<E: de::Error>(self, _: &str) -> Result<Self, E> {
            Ok(self)
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self, A::Error> {
            while seq.next_element::<Self>()?.is_some() {}
            Ok(self)
        }

        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self, A::Error> {
            while map.next_entry::<Self, Self>()?.is_some() {}
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, Generated, GeneratedStorage};
    use crate::model::column_type::{CatalogueType, ColumnType};
    use std::borrow::Cow;

    /// A column of the fixture's `consignment`, built with the defaults a test
    /// that is not about them does not want to restate.
    fn column<'a>(name: &'a str) -> Column<'a> {
        Column {
            name: Cow::Borrowed(name),
            table_name: Cow::Borrowed("consignment"),
            position: 1,
            column_type: ColumnType::decompose(&CatalogueType {
                column_type: "bigint(20) unsigned",
                data_type: "bigint",
                numeric_precision: Some(20),
                numeric_scale: Some(0),
                ..CatalogueType::default()
            }),
            nullable: false,
            default: None,
            comment: Cow::Borrowed(""),
            auto_increment: true,
            invisible: false,
            generated: None,
            on_update: None,
        }
    }

    #[test]
    fn fr_ctx_019_a_column_names_the_table_it_belongs_to() {
        // FR-CTX-019: a column reached through an index or a key must find its
        // own table without the template having carried a reference to it.
        assert_eq!(column("consignment_id").table_name, "consignment");
    }

    #[test]
    fn fr_cat_051_the_storage_kind_is_read_from_the_attribute_field_and_from_no_other() {
        // FR-CAT-051: the is-generated field reads `ALWAYS` for both kinds and
        // cannot be the source. These are the two values the attribute field
        // was observed to take over the fixture's nine generated columns.
        assert_eq!(
            GeneratedStorage::from_attribute("VIRTUAL GENERATED"),
            Some(GeneratedStorage::Virtual)
        );
        assert_eq!(
            GeneratedStorage::from_attribute("STORED GENERATED"),
            Some(GeneratedStorage::Stored)
        );
        assert_eq!(GeneratedStorage::from_attribute("ALWAYS"), None);
        assert_eq!(GeneratedStorage::from_attribute("auto_increment"), None);
        assert_eq!(GeneratedStorage::from_attribute(""), None);
    }

    #[test]
    fn fr_cat_051_the_attribute_spelling_round_trips() {
        // The spelling is contract surface once the attribute it reports
        // reaches the document, and the case of each value is the catalogue's.
        for storage in [GeneratedStorage::Virtual, GeneratedStorage::Stored] {
            assert_eq!(
                GeneratedStorage::from_attribute(storage.attribute()),
                Some(storage)
            );
        }
    }

    #[test]
    fn fr_cat_051_a_generated_column_carries_the_expression_and_the_kind_together() {
        // FR-CAT-051: one value, so neither half can be present alone. The
        // expression is the fixture's, as the catalogue rewrote it.
        let generated = Column {
            generated: Some(Generated {
                expression: Cow::Borrowed(
                    "round(`declared_value` * coalesce(`insurance_rate`,0),2)",
                ),
                storage: GeneratedStorage::Stored,
            }),
            ..column("insured_value")
        };

        let carried = generated
            .generated
            .as_ref()
            .expect("the column was built generated");

        assert_eq!(carried.storage, GeneratedStorage::Stored);
        assert!(carried.expression.contains("coalesce"));
    }

    #[test]
    fn fr_cat_027_the_auto_increment_a_column_carries_is_the_attribute_and_not_the_counter() {
        // FR-CAT-027 keeps the column attribute; FR-CAT-024 excludes the
        // table-level counter of the same name outright. The two are one word
        // and two different things, and only one of them is a field.
        assert!(column("consignment_id").auto_increment);
    }

    /// The decoding of a column, pinned against the derive it replaced
    /// (`FR-SCH-022`, `FR-CACHE-033`, `FR-RND-020`).
    ///
    /// Two callers observe what a decode answers, and each observes less than
    /// the whole error: the cache turns every failure into a miss, and
    /// `--context` reports the category of `serde_json` — a document that is
    /// not JSON, with the position the decoder stopped at, or one that is JSON
    /// and breaks the contract. So the contract pinned here is: the same
    /// documents accepted, into equal values, and every refusal in the same
    /// category, at the same position where the position is reported.
    mod decoding {
        use super::super::{Column, Generated, GeneratedStorage};
        use crate::model::column_default::ColumnDefault;
        use crate::model::column_type::{CatalogueType, ColumnType};
        use serde::Deserialize;
        use serde_json::error::Category;
        use std::borrow::Cow;

        /// The derive `Column` carried before its decoding was written by
        /// hand, attribute for attribute. It is the reference every case
        /// below is decided against, so the hand-written decoding is held to
        /// what the derive did rather than to what a test author expected.
        #[derive(Debug, Deserialize)]
        struct Derived<'a> {
            #[serde(borrow)]
            name: Cow<'a, str>,
            #[serde(borrow)]
            table_name: Cow<'a, str>,
            position: u64,
            #[serde(flatten)]
            column_type: ColumnType<'a>,
            nullable: bool,
            default: Option<ColumnDefault<'a>>,
            #[serde(borrow)]
            comment: Cow<'a, str>,
            auto_increment: bool,
            invisible: bool,
            generated: Option<Generated<'a>>,
            #[serde(borrow)]
            on_update: Option<Cow<'a, str>>,
        }

        /// What one decode answered, reduced to what a caller can observe.
        #[derive(Debug, PartialEq)]
        enum Outcome {
            /// Accepted, and re-emitted as these bytes.
            Accepted(String),
            /// Refused as data that breaks the contract.
            Data,
            /// Refused as text that is not JSON, at this line and column.
            NotJson(Category, usize, usize),
        }

        fn refusal(error: &serde_json::Error) -> Outcome {
            match error.classify() {
                Category::Data => Outcome::Data,
                other => Outcome::NotJson(other, error.line(), error.column()),
            }
        }

        /// What the decoding under test answers for `text`.
        fn current(text: &str) -> Outcome {
            match serde_json::from_str::<Column<'_>>(text) {
                Ok(column) => Outcome::Accepted(
                    serde_json::to_string(&column).expect("a decoded column serialises"),
                ),
                Err(error) => refusal(&error),
            }
        }

        /// What the reference derive answers for `text`, re-emitted through
        /// `Column` so that the two are compared field for field.
        fn derived(text: &str) -> Outcome {
            match serde_json::from_str::<Derived<'_>>(text) {
                Ok(read) => {
                    let column = Column {
                        name: read.name,
                        table_name: read.table_name,
                        position: read.position,
                        column_type: read.column_type,
                        nullable: read.nullable,
                        default: read.default,
                        comment: read.comment,
                        auto_increment: read.auto_increment,
                        invisible: read.invisible,
                        generated: read.generated,
                        on_update: read.on_update,
                    };
                    Outcome::Accepted(
                        serde_json::to_string(&column).expect("a decoded column serialises"),
                    )
                }
                Err(error) => refusal(&error),
            }
        }

        /// A column with every optional member present, including the member
        /// list, an escaped string and a generated expression.
        fn full() -> Column<'static> {
            Column {
                name: Cow::Borrowed("status"),
                table_name: Cow::Borrowed("consignment"),
                position: 7,
                column_type: ColumnType::decompose(&CatalogueType {
                    column_type: "enum('Draft','Booked''s','Loaded')",
                    data_type: "enum",
                    character_maximum_length: Some(8),
                    charset: Some("utf8mb4"),
                    collation: Some("utf8mb4_unicode_520_ci"),
                    ..CatalogueType::default()
                }),
                nullable: true,
                default: ColumnDefault::classify(Some("'Draft'")),
                comment: Cow::Borrowed("The \"state\" of a\nconsignment"),
                auto_increment: false,
                invisible: true,
                generated: Some(Generated {
                    expression: Cow::Borrowed("concat(`a`,'b')"),
                    storage: GeneratedStorage::Virtual,
                }),
                on_update: Some(Cow::Borrowed("on update current_timestamp()")),
            }
        }

        /// A column with every optional member absent.
        fn bare() -> Column<'static> {
            Column {
                default: None,
                generated: None,
                on_update: None,
                column_type: ColumnType::decompose(&CatalogueType {
                    column_type: "bigint(20) unsigned",
                    data_type: "bigint",
                    numeric_precision: Some(20),
                    numeric_scale: Some(0),
                    ..CatalogueType::default()
                }),
                ..full()
            }
        }

        /// The members of `column` as it is emitted, each as its key and its
        /// value's JSON text, in the emitted order.
        fn members(column: &Column<'_>) -> Vec<(String, String)> {
            let emitted = serde_json::to_string(column).expect("a column serialises");
            let value: serde_json::Value =
                serde_json::from_str(&emitted).expect("the emitted column is JSON");
            let serde_json::Value::Object(map) = value else {
                panic!("a column is emitted as an object");
            };
            map.into_iter()
                .map(|(key, value)| (key, value.to_string()))
                .collect()
        }

        /// One object from `members`, in the order given.
        fn object(members: &[(String, String)]) -> String {
            let inner: Vec<String> = members
                .iter()
                .map(|(key, value)| {
                    format!("{}:{value}", serde_json::to_string(key).expect("a key"))
                })
                .collect();
            format!("{{{}}}", inner.join(","))
        }

        /// The values every member is replaced with in turn: each JSON kind,
        /// the edges of the integer range, a lone surrogate, a number out of
        /// range, and three texts that are not JSON at all.
        const REPLACEMENTS: &[&str] = &[
            "null",
            "true",
            "false",
            "0",
            "1",
            "-1",
            "1.5",
            "18446744073709551615",
            "18446744073709551616",
            "1e400",
            "\"\"",
            "\"x\"",
            "\"\\u00e9\\\"q\"",
            "\"\\ud800\"",
            "[]",
            "[\"a\"]",
            "[\"a\",1]",
            "[null]",
            "[[\"a\"]]",
            "{}",
            "{\"kind\":\"null\"}",
            "{\"kind\":\"literal\",\"value\":\"1\"}",
            "{\"expression\":\"a\",\"storage\":\"STORED GENERATED\"}",
            "{\"a\":1e400}",
            "tru",
            "\"unterminated",
            "[1,",
        ];

        /// Every document the cases below are decided over.
        fn corpus() -> Vec<String> {
            let mut corpus = Vec::new();

            for column in [full(), bare()] {
                let members = members(&column);
                corpus.push(object(&members));

                let mut reversed = members.clone();
                reversed.reverse();
                corpus.push(object(&reversed));

                for at in 0..members.len() {
                    // Each member missing.
                    let mut missing = members.clone();
                    missing.remove(at);
                    corpus.push(object(&missing));

                    // Each member twice, adjacent and apart, with the same
                    // value and with another.
                    let mut twice = members.clone();
                    twice.insert(at, members[at].clone());
                    corpus.push(object(&twice));
                    let mut apart = members.clone();
                    apart.push(members[at].clone());
                    corpus.push(object(&apart));
                    let mut other = members.clone();
                    other.push((members[at].0.clone(), "null".to_owned()));
                    corpus.push(object(&other));

                    for replacement in REPLACEMENTS {
                        // Each member's value replaced.
                        let mut replaced = members.clone();
                        replaced[at].1 = (*replacement).to_owned();
                        corpus.push(object(&replaced));

                        // The same, with text that is not JSON further on in
                        // the same object: which of the two is reported is
                        // decided by when the value is checked.
                        let mut then_broken = replaced.clone();
                        then_broken.push(("zzz".to_owned(), "tru".to_owned()));
                        corpus.push(object(&then_broken));
                    }
                }

                for replacement in REPLACEMENTS {
                    // A member no field names, first and last.
                    let mut first = members.clone();
                    first.insert(0, ("unknown".to_owned(), (*replacement).to_owned()));
                    corpus.push(object(&first));
                    let mut last = members.clone();
                    last.push(("unknown".to_owned(), (*replacement).to_owned()));
                    corpus.push(object(&last));
                }

                // A key spelled with an escape.
                let mut escaped = object(&members);
                escaped = escaped.replacen("\"name\"", "\"na\\u006de\"", 1);
                corpus.push(escaped);
            }

            for other in [
                "[]",
                "[{}]",
                "null",
                "1",
                "\"x\"",
                "{}",
                "",
                "{",
                "{\"name\"",
            ] {
                corpus.push((*other).to_owned());
            }

            corpus
        }

        #[test]
        fn every_document_of_the_corpus_is_decoded_as_the_derive_decoded_it() {
            let corpus = corpus();
            assert!(
                corpus.len() > 1_000,
                "the corpus is {} documents",
                corpus.len()
            );

            for text in &corpus {
                assert_eq!(current(text), derived(text), "for {text}");
            }
        }

        #[test]
        fn a_column_reads_back_equal_in_either_member_order() {
            for column in [full(), bare()] {
                let members = members(&column);
                let mut reversed = members.clone();
                reversed.reverse();

                for text in [object(&members), object(&reversed)] {
                    let read: Column<'_> =
                        serde_json::from_str(&text).expect("the emitted column reads back");
                    assert_eq!(read, column);
                }
            }
        }

        #[test]
        fn the_members_a_column_requires_and_the_members_it_does_not() {
            // Pinned before the decoding was written by hand: eight members
            // are required, and the eleven that are optional read as absent.
            let members = members(&full());
            let mut required = Vec::new();

            for at in 0..members.len() {
                let mut missing = members.clone();
                missing.remove(at);
                if current(&object(&missing)) == Outcome::Data {
                    required.push(members[at].0.clone());
                }
            }
            required.sort();

            assert_eq!(
                required,
                [
                    "auto_increment",
                    "column_type",
                    "comment",
                    "invisible",
                    "name",
                    "nullable",
                    "position",
                    "table_name",
                ]
            );
        }

        #[test]
        fn a_member_given_twice_is_refused_and_an_unknown_one_is_ignored() {
            let members = members(&full());

            for at in 0..members.len() {
                let mut apart = members.clone();
                apart.push(members[at].clone());
                assert_eq!(current(&object(&apart)), Outcome::Data, "{}", members[at].0);
            }

            let mut unknown = members.clone();
            unknown.push(("unknown".to_owned(), "{\"a\":[1,{\"b\":null}]}".to_owned()));
            assert_eq!(current(&object(&unknown)), current(&object(&members)));
        }

        #[test]
        fn a_type_member_of_the_wrong_kind_is_reported_after_the_rest_of_the_object() {
            // The derive buffered the nine type members and decoded them at
            // the end of the object, so text that is not JSON further on is
            // what a decode reports; a column's own members are decoded as
            // they are met.
            let members = members(&full());
            let replaced = |key: &str| {
                let mut replaced = members.clone();
                for member in &mut replaced {
                    if member.0 == key {
                        member.1 = "\"x\"".to_owned();
                    }
                }
                replaced.push(("zzz".to_owned(), "tru".to_owned()));
                object(&replaced)
            };

            assert!(matches!(
                current(&replaced("length")),
                Outcome::NotJson(Category::Syntax, 1, _)
            ));
            assert_eq!(current(&replaced("position")), Outcome::Data);
        }

        #[test]
        fn a_value_out_of_range_under_an_unknown_member_is_not_json() {
            let mut members = members(&bare());
            members.push(("unknown".to_owned(), "1e400".to_owned()));

            assert!(matches!(
                current(&object(&members)),
                Outcome::NotJson(Category::Syntax, 1, _)
            ));
        }

        #[test]
        fn a_column_is_an_object_and_nothing_else() {
            for text in ["[]", "null", "1", "\"x\""] {
                assert_eq!(current(text), Outcome::Data, "for {text}");
            }
        }
    }
}
