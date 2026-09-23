//! What a template can use, as help states it: the item values of
//! `FR-ENV-047`, indexed by name, and the context variables of `FR-HELP-032`,
//! indexed by variable.
//!
//! `FR-HELP-022` places both in the typed table, beside the purposes, the
//! examples and the exit codes, and for the same reason: each is written once
//! and read by two channels — the text help of `tpl render`, per
//! `FR-HELP-033`, and the JSON document, per `FR-ENV-005` and `FR-HELP-032` —
//! and neither channel is derived from the other.
//!
//! # What this module does not own
//!
//! **The names.** Which filters, tests and functions exist is `render/`'s:
//! `FR-ENV-005` derives the published arrays from the registrations the
//! environment actually performs, so both consumers walk
//! [`crate::render::REGISTERED_FILTERS`] and its three siblings and look each
//! name up here. A name registered and not described here is a defect the
//! suite catches, and so is a description of a name nothing registers.
//!
//! **The behaviour.** Every sentence here restates what the owning requirement
//! fixes, and states nothing it does not fix, per `FR-ENV-047`. For an
//! inherited filter the owner is the pinned engine of `FR-ENV-003`, and each
//! row was checked against it: an argument the engine takes only by keyword
//! is written `name=…` in the signature, or `name=default` where it has one,
//! because a template that passes it by position fails.

use serde::Serialize;

/// The type of an operand or of an argument, from the closed vocabulary of
/// `FR-ENV-047`.
///
/// Only the members some item uses are declared; `table`, `view` and
/// `routine` belong to the vocabulary and no item of the surface takes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Kind {
    /// Any value the context can hold.
    Any,
    /// A string.
    String,
    /// An integer.
    Integer,
    /// A boolean.
    Boolean,
    /// A list.
    List,
    /// An object.
    Object,
    /// A column object of the context.
    Column,
}

/// One argument a template passes to a filter, a test or a function
/// (`FR-ENV-047`).
///
/// The four members are the four keys that requirement fixes, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct Parameter {
    /// The argument's name, as the engine spells it.
    pub(crate) name: &'static str,

    /// The type of value it takes.
    #[serde(rename = "type")]
    pub(crate) kind: Kind,

    /// Whether a template must pass it.
    pub(crate) required: bool,

    /// The default as a template would write it, or `null` where the argument
    /// is required or has no default.
    pub(crate) default: Option<&'static str>,
}

/// One filter, test or function of the template surface (`FR-ENV-047`).
///
/// The five members are the five keys that requirement fixes, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct Item {
    /// The name a template writes.
    pub(crate) name: &'static str,

    /// The form a template writes to use it.
    pub(crate) signature: &'static str,

    /// The type accepted on the left of a filter or a test, or `null` for a
    /// function.
    pub(crate) operand: Option<Kind>,

    /// The arguments, in positional order; empty where there are none.
    pub(crate) arguments: &'static [Parameter],

    /// One sentence stating what it returns or does.
    pub(crate) purpose: &'static str,
}

/// One top-level context variable (`FR-HELP-032`).
///
/// The four members are the four keys that requirement fixes, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct Variable {
    /// The variable's name.
    pub(crate) name: &'static str,

    /// `object` or `string`.
    #[serde(rename = "type")]
    pub(crate) kind: Kind,

    /// The object flag whose presence binds it, or `null` where every render
    /// carries it.
    pub(crate) bound_by: Option<&'static str>,

    /// One sentence stating what it holds.
    pub(crate) purpose: &'static str,
}

/// A required argument.
const fn required(name: &'static str, kind: Kind) -> Parameter {
    Parameter {
        name,
        kind,
        required: true,
        default: None,
    }
}

/// An optional argument, with the default a template would write, if any.
const fn optional(name: &'static str, kind: Kind, default: Option<&'static str>) -> Parameter {
    Parameter {
        name,
        kind,
        required: false,
        default,
    }
}

/// A filter that takes a string and no argument.
const fn text_filter(name: &'static str, signature: &'static str, purpose: &'static str) -> Item {
    Item {
        name,
        signature,
        operand: Some(Kind::String),
        arguments: &[],
        purpose,
    }
}

/// A test, which takes a column and no argument (`FR-ENV-040`).
const fn column_test(name: &'static str, signature: &'static str, purpose: &'static str) -> Item {
    Item {
        name,
        signature,
        operand: Some(Kind::Column),
        arguments: &[],
        purpose,
    }
}

/// The one argument of a lookup function.
const NAME: &[Parameter] = &[required("name", Kind::String)];

/// A lookup function of one name argument (`FR-ENV-020`).
const fn lookup(name: &'static str, signature: &'static str, purpose: &'static str) -> Item {
    Item {
        name,
        signature,
        operand: None,
        arguments: NAME,
        purpose,
    }
}

/// Every filter help describes: the registered ones of `FR-ENV-006` and
/// `FR-ENV-007`, then the inherited ones of `FR-ENV-018`.
const FILTERS: &[Item] = &[
    text_filter(
        "pascal",
        "value | pascal",
        "Joins the words of the string in PascalCase: order_items becomes OrderItems.",
    ),
    text_filter(
        "camel",
        "value | camel",
        "Joins the words of the string in camelCase: order_items becomes orderItems.",
    ),
    text_filter(
        "snake",
        "value | snake",
        "Joins the words of the string in snake_case: OrderItems becomes order_items.",
    ),
    text_filter(
        "upper_snake",
        "value | upper_snake",
        "Joins the words of the string in UPPER_SNAKE_CASE: order_items becomes ORDER_ITEMS.",
    ),
    text_filter(
        "kebab",
        "value | kebab",
        "Joins the words of the string in kebab-case: order_items becomes order-items.",
    ),
    text_filter(
        "quote",
        "value | quote",
        "Quotes the string as a MariaDB identifier in backticks, doubling any backtick inside it.",
    ),
    Item {
        name: "sql_type",
        signature: "value | sql_type",
        operand: Some(Kind::Column),
        arguments: &[],
        purpose: "Returns the column's data_type, such as varchar, or null when tpl does not \
                  recognise the type; any operand but a column fails the render.",
    },
    Item {
        name: "json",
        signature: "value | json",
        operand: Some(Kind::Any),
        arguments: &[],
        purpose: "Writes the value as compact JSON on one line.",
    },
    Item {
        name: "indent",
        signature: "value | indent(n)",
        operand: Some(Kind::String),
        arguments: &[required("n", Kind::Integer)],
        purpose: "Prefixes every line after the first with n spaces, leaving empty lines empty.",
    },
    Item {
        name: "comment",
        signature: "value | comment(prefix)",
        operand: Some(Kind::String),
        arguments: &[required("prefix", Kind::String)],
        purpose: "Prefixes every line with prefix and one space, as in comment(\"//\").",
    },
    Item {
        name: "escape",
        signature: "value | escape(target)",
        operand: Some(Kind::String),
        arguments: &[required("target", Kind::String)],
        purpose: "Replaces & < > \" and ' with entities for target \"html\" or \"xml\"; any other \
                  target fails the render.",
    },
    Item {
        name: "default",
        signature: "value | default(fallback, boolean)",
        operand: Some(Kind::Any),
        arguments: &[
            optional("fallback", Kind::Any, Some("\"\"")),
            optional("boolean", Kind::Boolean, Some("false")),
        ],
        purpose: "Returns fallback when the value is undefined, and also when it is empty or \
                  false if boolean is true.",
    },
    Item {
        name: "join",
        signature: "value | join(separator)",
        operand: Some(Kind::List),
        arguments: &[optional("separator", Kind::String, Some("\"\""))],
        purpose: "Joins the items into one string, with separator between them.",
    },
    Item {
        name: "length",
        signature: "value | length",
        operand: Some(Kind::Any),
        arguments: &[],
        purpose: "Returns the number of items of a list or an object, or of characters of a \
                  string.",
    },
    Item {
        name: "map",
        signature: "value | map(attribute=…)",
        operand: Some(Kind::List),
        arguments: &[required("attribute", Kind::String)],
        purpose: "Returns the named attribute of every item, as in map(attribute=\"name\").",
    },
    Item {
        name: "select",
        signature: "value | select(test)",
        operand: Some(Kind::List),
        arguments: &[optional("test", Kind::String, None)],
        purpose: "Returns the items that pass the named test, or the true items when no test is \
                  named.",
    },
    Item {
        name: "reject",
        signature: "value | reject(test)",
        operand: Some(Kind::List),
        arguments: &[optional("test", Kind::String, None)],
        purpose: "Returns the items that fail the named test, or the false items when no test is \
                  named.",
    },
    Item {
        name: "first",
        signature: "value | first",
        operand: Some(Kind::List),
        arguments: &[],
        purpose: "Returns the first item, or undefined when the list is empty.",
    },
    Item {
        name: "last",
        signature: "value | last",
        operand: Some(Kind::List),
        arguments: &[],
        purpose: "Returns the last item, or undefined when the list is empty.",
    },
    Item {
        name: "reverse",
        signature: "value | reverse",
        operand: Some(Kind::Any),
        arguments: &[],
        purpose: "Returns the list, or the string, in reverse order.",
    },
    Item {
        name: "sort",
        signature: "value | sort(attribute=…, reverse=false, case_sensitive=false)",
        operand: Some(Kind::List),
        arguments: &[
            optional("attribute", Kind::String, None),
            optional("reverse", Kind::Boolean, Some("false")),
            optional("case_sensitive", Kind::Boolean, Some("false")),
        ],
        purpose: "Returns the list sorted in ascending order, by the named attribute when \
                  attribute is given and in descending order when reverse is true.",
    },
    Item {
        name: "trim",
        signature: "value | trim(chars)",
        operand: Some(Kind::String),
        arguments: &[optional("chars", Kind::String, None)],
        purpose: "Removes leading and trailing whitespace, or leading and trailing runs of chars \
                  when chars is given.",
    },
    text_filter(
        "upper",
        "value | upper",
        "Converts the string to upper case.",
    ),
    text_filter(
        "lower",
        "value | lower",
        "Converts the string to lower case.",
    ),
    Item {
        name: "replace",
        signature: "value | replace(from, to)",
        operand: Some(Kind::String),
        arguments: &[required("from", Kind::String), required("to", Kind::String)],
        purpose: "Replaces every occurrence of from with to.",
    },
];

/// Every test help describes, all registered by `FR-ENV-014`.
const TESTS: &[Item] = &[
    column_test(
        "nullable",
        "value is nullable",
        "True when the column admits NULL.",
    ),
    column_test(
        "primary_key",
        "value is primary_key",
        "True when the column is part of its table's primary key; fails the render when that \
         table is not in the context.",
    ),
    column_test(
        "auto_increment",
        "value is auto_increment",
        "True when the column is AUTO_INCREMENT.",
    ),
    column_test(
        "unique",
        "value is unique",
        "True when a unique index of its table includes the column; fails the render when that \
         table is not in the context.",
    ),
    column_test(
        "numeric",
        "value is numeric",
        "True when the column's data_type is a numeric type.",
    ),
    column_test(
        "temporal",
        "value is temporal",
        "True when the column's data_type is a date or time type.",
    ),
    column_test(
        "textual",
        "value is textual",
        "True when the column's data_type is a character-string type.",
    ),
];

/// Every global function help describes, all registered by `FR-ENV-020`.
const FUNCTIONS: &[Item] = &[
    lookup(
        "table",
        "table(name)",
        "Returns the table of that name; using the result fails the render when there is none.",
    ),
    lookup(
        "view",
        "view(name)",
        "Returns the view of that name; using the result fails the render when there is none.",
    ),
    lookup(
        "routine",
        "routine(name)",
        "Returns the first routine of that name; using the result fails the render when there \
         is none.",
    ),
    Item {
        name: "column",
        signature: "column(table, name)",
        operand: None,
        arguments: &[
            required("table", Kind::String),
            required("name", Kind::String),
        ],
        purpose: "Returns the column name of the table named table; using the result fails the \
                  render when either is missing.",
    },
    Item {
        name: "fail",
        signature: "fail(message)",
        operand: None,
        arguments: &[required("message", Kind::String)],
        purpose: "Stops the render with exit 65, reporting message as the error.",
    },
];

/// The seven top-level context variables of `FR-RND-023`, in the order
/// `FR-HELP-032` fixes.
pub(crate) const VARIABLES: &[Variable] = &[
    Variable {
        name: "database",
        kind: Kind::Object,
        bound_by: None,
        purpose: "The whole database: name, charset, collation, server, tables, views and \
                  routines.",
    },
    Variable {
        name: "table",
        kind: Kind::Object,
        bound_by: Some("--table"),
        purpose: "The table named by --table, with its columns, indexes, foreign keys and \
                  triggers; undefined without --table.",
    },
    Variable {
        name: "view",
        kind: Kind::Object,
        bound_by: Some("--view"),
        purpose: "The view named by --view, with its definition; undefined without --view.",
    },
    Variable {
        name: "routine",
        kind: Kind::Object,
        bound_by: Some("--routine"),
        purpose: "The routine named by --routine, with its parameters and body; undefined \
                  without --routine.",
    },
    Variable {
        name: "vars",
        kind: Kind::Object,
        bound_by: None,
        purpose: "The --set values: --set title=Orders makes vars.title the string Orders; empty \
                  without --set.",
    },
    Variable {
        name: "tpl",
        kind: Kind::Object,
        bound_by: None,
        purpose: "The running tpl: tpl.version is its version string.",
    },
    Variable {
        name: "now",
        kind: Kind::String,
        bound_by: None,
        purpose: "The render time in UTC, as in 2026-09-10T08:14:22Z.",
    },
];

/// The three kinds of item a template can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Family {
    /// A filter, written after `|`.
    Filter,
    /// A test, written after `is`.
    Test,
    /// A global function, called by name.
    Function,
}

/// The description of the `family` item called `name`, or [`None`] where help
/// describes no such item.
///
/// A linear scan, for the reason [`super::entry`] gives: the tables are small
/// and ordered, and no unordered map appears on this path.
pub(crate) fn item(family: Family, name: &str) -> Option<&'static Item> {
    let listed = match family {
        Family::Filter => FILTERS,
        Family::Test => TESTS,
        Family::Function => FUNCTIONS,
    };

    listed.iter().find(|item| item.name == name)
}

/// The descriptions of `names`, in their order.
///
/// # Errors
///
/// Returns [`crate::error::Error::InternalInvariant`] where a name carries no
/// description, which the suite holds impossible.
pub(crate) fn items(
    family: Family,
    names: &[&str],
) -> Result<Vec<&'static Item>, crate::error::Error> {
    let mut described = Vec::with_capacity(names.len());

    for name in names {
        let found = item(family, name);

        crate::error::ensure_invariant(
            found.is_some(),
            "every registered or inherited name carries a description in the help table",
        )?;

        described.extend(found);
    }

    Ok(described)
}

#[cfg(test)]
mod tests {
    use super::{FILTERS, FUNCTIONS, Family, Kind, TESTS, VARIABLES, item};
    use crate::render::{
        INHERITED_FILTERS, REGISTERED_FILTERS, REGISTERED_FUNCTIONS, REGISTERED_TESTS,
    };

    /// Every family, with the names it publishes and the rows help holds.
    fn families() -> [(Family, Vec<&'static str>, &'static [super::Item]); 3] {
        [
            (
                Family::Filter,
                REGISTERED_FILTERS
                    .iter()
                    .chain(INHERITED_FILTERS)
                    .copied()
                    .collect(),
                FILTERS,
            ),
            (Family::Test, REGISTERED_TESTS.to_vec(), TESTS),
            (Family::Function, REGISTERED_FUNCTIONS.to_vec(), FUNCTIONS),
        ]
    }

    #[test]
    fn fr_env_047_every_published_name_has_an_item_and_no_item_names_nothing() {
        for (family, names, rows) in families() {
            for name in &names {
                assert!(item(family, name).is_some(), "{name} has no item");
            }

            for row in rows {
                assert!(
                    names.contains(&row.name),
                    "{} is described and neither registered nor inherited",
                    row.name
                );
            }

            assert_eq!(names.len(), rows.len(), "{family:?} holds a duplicate row");
        }
    }

    #[test]
    fn fr_env_047_every_signature_has_the_form_of_its_family_and_names_its_arguments() {
        for (family, _, rows) in families() {
            for row in rows {
                let form = match family {
                    Family::Filter => format!("value | {}", row.name),
                    Family::Test => format!("value is {}", row.name),
                    Family::Function => row.name.to_owned(),
                };

                assert!(row.signature.starts_with(&form), "{}", row.signature);
                assert_eq!(
                    row.operand.is_none(),
                    family == Family::Function,
                    "{}: only a function has no operand",
                    row.name
                );

                if row.arguments.is_empty() {
                    assert!(!row.signature.contains('('), "{}", row.signature);
                } else {
                    // Each argument is written by its name alone, or as
                    // `name=…` or `name=default` where the engine takes it
                    // only by keyword, in the order `arguments` lists them.
                    let open = format!("{}(", row.name);
                    let written = row
                        .signature
                        .split_once(&open)
                        .and_then(|(_, rest)| rest.strip_suffix(')'))
                        .unwrap_or_else(|| panic!("{} writes no argument list", row.signature));
                    let listed: Vec<&str> = written
                        .split(", ")
                        .map(|argument| argument.split('=').next().unwrap_or(argument))
                        .collect();
                    let declared: Vec<&str> =
                        row.arguments.iter().map(|argument| argument.name).collect();

                    assert_eq!(listed, declared, "{}", row.signature);

                    for (form, argument) in written.split(", ").zip(row.arguments) {
                        if let Some((_, value)) = form.split_once('=') {
                            let expected = argument.default.unwrap_or("…");
                            assert_eq!(value, expected, "{}", row.signature);
                        }
                    }
                }

                for argument in row.arguments {
                    assert!(
                        !(argument.required && argument.default.is_some()),
                        "{}: a required argument carries a default",
                        row.name
                    );
                }

                assert!(row.purpose.ends_with('.'), "{}", row.purpose);
            }
        }
    }

    #[test]
    fn fr_env_040_every_test_and_sql_type_take_a_column() {
        for row in TESTS {
            assert_eq!(row.operand, Some(Kind::Column), "{}", row.name);
        }

        assert_eq!(
            item(Family::Filter, "sql_type").map(|row| row.operand),
            Some(Some(Kind::Column))
        );
    }

    #[test]
    fn fr_env_037_038_044_the_three_code_filters_take_a_required_argument() {
        for name in ["indent", "comment", "escape"] {
            let row = item(Family::Filter, name).expect("described");

            assert_eq!(row.arguments.len(), 1, "{name}");
            assert!(row.arguments[0].required, "{name}");
            assert_eq!(row.arguments[0].default, None, "{name}");
        }
    }

    #[test]
    fn fr_help_032_the_variables_are_the_seven_of_the_requirement_in_its_order() {
        let names: Vec<&str> = VARIABLES.iter().map(|variable| variable.name).collect();

        assert_eq!(
            names,
            ["database", "table", "view", "routine", "vars", "tpl", "now"]
        );

        for variable in VARIABLES {
            let flag = format!("--{}", variable.name);
            let bound = ["table", "view", "routine"].contains(&variable.name);

            assert_eq!(variable.bound_by, bound.then_some(flag.as_str()));
            assert!(
                matches!(variable.kind, Kind::Object | Kind::String),
                "{}",
                variable.name
            );
            assert!(variable.purpose.ends_with('.'), "{}", variable.purpose);
        }
    }

    #[test]
    fn fr_help_014_no_sentence_of_the_surface_cites_a_requirement() {
        const PREFIXES: [&str; 8] = ["FR-", "NFR-", "BR-", "UC-", "OD-", "ADR-", "OQ-", "DIV-"];

        let sentences = FILTERS
            .iter()
            .chain(TESTS)
            .chain(FUNCTIONS)
            .map(|row| row.purpose)
            .chain(VARIABLES.iter().map(|variable| variable.purpose));

        for sentence in sentences {
            for prefix in PREFIXES {
                assert!(!sentence.contains(prefix), "{sentence}");
            }
        }
    }
}
