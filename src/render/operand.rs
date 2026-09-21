//! What a registered filter or test was handed, and the refusal when it is the
//! wrong thing.
//!
//! One rule governs the whole module, and it is `BR-SEM-004`'s: a render that
//! cannot be certain of the right output fails rather than producing plausible
//! output quietly. `FR-SEM-005` through `FR-SEM-009` state it for the two
//! grammatical classes, and neither is allowed the escape that nearly every
//! template engine takes by default:
//!
//! | Requirement | What it forbids |
//! |---|---|
//! | `FR-SEM-005`, `FR-SEM-007`, `FR-ENV-040` | A test answering `false` for an operand it does not accept |
//! | `FR-SEM-008`, `FR-ENV-034`, `FR-ENV-039` | A filter returning an empty string, or anything else, for an operand it does not accept |
//! | `FR-SEM-009` | Coercing a value to a string before applying a filter that requires one |
//! | `FR-SEM-006` | A message that does not name the test, the type received and the location |
//!
//! The location is the engine's to attach — it holds the template, the line
//! and the byte range — so every refusal here is a [`minijinja::Error`] and
//! reaches the caller as the `chain` of `FR-ERR-011` through
//! [`super::fault`].
//!
//! # What makes a value a column
//!
//! Eight of the registered names accept a column object and nothing else: the
//! `sql_type` filter of `FR-ENV-039` and the seven tests of `FR-ENV-014`. The
//! model states no discriminant for one — a column is plain data, per
//! `crate::model` — so the operand is identified by the two fields no other
//! object of the document carries together: `table_name`, which `FR-CTX-019`
//! puts on every column and on nothing else, and `column_type`, which
//! `FR-CTX-014` puts on every column.
//!
//! *Rejected: deserialising the operand into [`crate::model::column::Column`].*
//! It is the strongest identification available and it costs an owned copy of
//! every string of the column on **every** call, because
//! [`minijinja::Value`] deserialises borrowing nothing. `{% if col is
//! nullable %}` inside a loop over a table's columns would allocate a column
//! per iteration to read one boolean, which is what the project's
//! minimal-allocation rule refuses. Reading the one or two attributes each
//! name needs allocates nothing.
//!
//! *Rejected: accepting any map.* `FR-ENV-040` says "a column object and
//! nothing else", and a table is a map too.

use std::sync::Arc;

use minijinja::value::ValueKind;
use minijinja::{Error, ErrorKind, Value};

/// The attribute `FR-CTX-019` puts on every column and on no other object.
const TABLE_NAME: &str = "table_name";

/// The attribute `FR-CTX-014` puts on every column.
const COLUMN_TYPE: &str = "column_type";

/// The column's own name.
const NAME: &str = "name";

/// The normalised type name of `FR-CTX-015`, inside the decomposed type.
const DATA_TYPE: &str = "data_type";

/// Which grammatical class refused, so that the message names it as the
/// template author wrote it (`FR-SEM-006`, `FR-SEM-008`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    /// A filter, applied with `|`.
    Filter,
    /// A test, applied with `is`.
    Test,
}

impl Role {
    /// The word the message calls this class by.
    const fn word(self) -> &'static str {
        match self {
            Self::Filter => "filter",
            Self::Test => "test",
        }
    }
}

/// The refusal of `FR-SEM-005`, `FR-SEM-006` and `FR-SEM-008`.
///
/// It names the class, the name, what the name accepts and the type received.
/// The location is added by the engine, which is the only party that knows it.
pub(super) fn refused(role: Role, name: &str, accepts: &str, given: &Value) -> Error {
    Error::new(
        ErrorKind::InvalidOperation,
        format!(
            "the {} '{name}' accepts {accepts}, and was given a value of type {}",
            role.word(),
            given.kind()
        ),
    )
}

/// The string operand of `FR-SEM-008`, with no coercion (`FR-SEM-009`).
///
/// # Errors
///
/// Returns the refusal of [`refused`] for every value that is not a string,
/// including a number, which `FR-SEM-009` names by example.
pub(super) fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str, Error> {
    value
        .as_str()
        .ok_or_else(|| refused(Role::Filter, name, "a string", value))
}

/// A column operand, identified and read without being copied.
///
/// It borrows the value the engine handed the filter or the test, and reads
/// each attribute only when a caller asks for it, so a test that answers from
/// one boolean reads one attribute.
#[derive(Debug, Clone, Copy)]
pub(super) struct Column<'a> {
    /// The value itself, already established to be a column.
    value: &'a Value,

    /// Which class is asking, carried so that a malformed column is reported
    /// in the same words as a wrong one.
    role: Role,

    /// The registered name that is asking.
    name: &'a str,
}

impl<'a> Column<'a> {
    /// Accepts `value` as a column, or refuses it (`FR-ENV-039`,
    /// `FR-ENV-040`).
    ///
    /// # Errors
    ///
    /// Returns the refusal of [`refused`] where the value is not a map, or is
    /// a map that carries neither of the two attributes a column is identified
    /// by.
    pub(super) fn of(value: &'a Value, role: Role, name: &'a str) -> Result<Self, Error> {
        let column = Self { value, role, name };

        if value.kind() != ValueKind::Map
            || !column.carries(TABLE_NAME)
            || !column.carries(COLUMN_TYPE)
        {
            return Err(refused(role, name, "a column object", value));
        }

        Ok(column)
    }

    /// The column's own name.
    ///
    /// # Errors
    ///
    /// Returns the refusal of [`Column::malformed`] where the attribute is
    /// absent or is not a string.
    pub(super) fn name(&self) -> Result<Arc<str>, Error> {
        self.string(NAME)
    }

    /// The table the column belongs to (`FR-CTX-019`).
    ///
    /// # Errors
    ///
    /// Returns the refusal of [`Column::malformed`] where the attribute is
    /// absent or is not a string.
    pub(super) fn table_name(&self) -> Result<Arc<str>, Error> {
        self.string(TABLE_NAME)
    }

    /// The normalised type name of `FR-CTX-015`, or `null` under
    /// `FR-CTX-018`.
    ///
    /// The model carries the eight decomposed parts inside the column's
    /// `column_type`, per `crate::model::column_type`, so this is the one
    /// attribute that is read one level down.
    ///
    /// # Errors
    ///
    /// Returns the refusal of [`Column::malformed`] where the decomposed type
    /// carries no `data_type` at all, which is a column no read of a supported
    /// series produces.
    pub(super) fn data_type(&self) -> Result<Value, Error> {
        let decomposed = self.attribute(COLUMN_TYPE);

        if decomposed.kind() != ValueKind::Map {
            return Err(self.malformed(COLUMN_TYPE));
        }

        let data_type = decomposed
            .get_attr(DATA_TYPE)
            .map_err(|_| self.malformed(DATA_TYPE))?;

        if data_type.is_undefined() {
            return Err(self.malformed(DATA_TYPE));
        }

        Ok(data_type)
    }

    /// A boolean attribute of the column, such as the nullability of
    /// `FR-ENV-041` or the auto-increment attribute of `FR-CAT-027`.
    ///
    /// # Errors
    ///
    /// Returns the refusal of [`Column::malformed`] where the attribute is
    /// absent or is not a boolean. It is never answered as `false`, which
    /// `FR-SEM-007` forbids.
    pub(super) fn flag(&self, key: &str) -> Result<bool, Error> {
        let value = self.attribute(key);

        if value.kind() == ValueKind::Bool {
            Ok(value.is_true())
        } else {
            Err(self.malformed(key))
        }
    }

    /// Whether the value carries `key` at all.
    fn carries(&self, key: &str) -> bool {
        !self.attribute(key).is_undefined()
    }

    /// One attribute, or `undefined` where the value does not carry it.
    fn attribute(&self, key: &str) -> Value {
        self.value.get_attr(key).unwrap_or(Value::UNDEFINED)
    }

    /// A string attribute.
    fn string(&self, key: &str) -> Result<Arc<str>, Error> {
        self.attribute(key)
            .to_str()
            .ok_or_else(|| self.malformed(key))
    }

    /// A column that identified as one and does not carry what the name needs.
    ///
    /// It is reported rather than answered around, for the reason
    /// `FR-SEM-007` gives: the question could not be asked, and a `false`
    /// would say it was asked and answered.
    fn malformed(&self, key: &str) -> Error {
        Error::new(
            ErrorKind::InvalidOperation,
            format!(
                "the {} '{}' was given a column that carries no usable '{key}'",
                self.role.word(),
                self.name
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, Role, text};
    use crate::render::fixture;
    use minijinja::Value;

    #[test]
    fn fr_sem_009_a_string_operand_is_not_coerced() {
        // FR-SEM-009: `{{ 42 | snake }}` fails. The message names the filter
        // and the type received, per FR-SEM-008.
        let condition = text(&Value::from(42), "snake").expect_err("a number is not a string");
        let message = condition.to_string();

        assert!(message.contains("snake"), "{message}");
        assert!(message.contains("number"), "{message}");
    }

    #[test]
    fn fr_sem_008_a_string_operand_is_accepted_unchanged() {
        let value = Value::from("order_items");

        assert_eq!(
            text(&value, "snake").expect("it is a string"),
            "order_items"
        );
    }

    #[test]
    fn fr_env_040_a_column_is_identified_and_nothing_else_is() {
        // FR-ENV-040: a column object and nothing else. The four rejected
        // operands are four of the six the requirement enumerates.
        let column = fixture::column("consignment_id");

        assert!(Column::of(&column, Role::Test, "nullable").is_ok());

        for other in [
            fixture::table(),
            Value::from("consignment"),
            Value::from(1),
            Value::from(()),
        ] {
            let condition =
                Column::of(&other, Role::Test, "nullable").expect_err("it is not a column object");

            assert!(condition.to_string().contains("nullable"));
        }
    }

    #[test]
    fn fr_ctx_015_the_normalised_type_name_is_read_from_the_decomposed_type() {
        // FR-CTX-015 and FR-ENV-039: the value of the column's `data_type`,
        // exactly as the model carries it.
        let column = fixture::column("consignment_id");
        let column = Column::of(&column, Role::Filter, "sql_type").expect("it is a column");

        assert_eq!(
            column.data_type().expect("the type is decomposed").as_str(),
            Some("bigint")
        );
    }

    #[test]
    fn fr_ctx_018_an_unrecognised_type_reads_as_null_rather_than_as_absence() {
        // FR-CTX-018 nulls every part of an unrecognised type, and
        // FR-ENV-039's third row carries that through the filter.
        let column = fixture::column_of_unrecognised_type();
        let column = Column::of(&column, Role::Filter, "sql_type").expect("it is a column");

        assert!(
            column
                .data_type()
                .expect("the part is null, not absent")
                .is_none()
        );
    }

    #[test]
    fn fr_sem_007_a_column_without_the_attribute_is_reported_rather_than_answered() {
        // FR-SEM-007: never `false` for a question that could not be asked.
        let column = fixture::column_without("nullable");
        let column = Column::of(&column, Role::Test, "nullable").expect("it identifies as one");

        let condition = column
            .flag("nullable")
            .expect_err("the attribute is absent");

        assert!(condition.to_string().contains("nullable"));
    }
}
