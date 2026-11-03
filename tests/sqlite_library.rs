//! The SQLite library the build links (`FR-LITE-001`).
//!
//! `libsqlite3-sys` compiles its bundled source with
//! `-DSQLITE_ENABLE_LOAD_EXTENSION=1`; `.cargo/config.toml` adds the options
//! that undo it and omit the loader. These bodies read back what the build
//! produced, through the library's own answers, so that a build which lost
//! that setting fails here rather than shipping.

use rusqlite::Connection;

/// The rows `PRAGMA compile_options` returns, one option per row.
fn compile_options(connection: &Connection) -> Vec<String> {
    let mut statement = connection
        .prepare("PRAGMA compile_options")
        .expect("the pragma prepares");

    statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("the pragma runs")
        .collect::<Result<_, _>>()
        .expect("every row is one text value")
}

#[test]
fn fr_lite_001_the_library_is_built_without_extension_loading() {
    let connection = Connection::open_in_memory().expect("an in-memory database opens");
    let options = compile_options(&connection);

    assert!(
        options.iter().any(|option| option == "OMIT_LOAD_EXTENSION"),
        "OMIT_LOAD_EXTENSION is not among {options:?}"
    );
    assert!(
        !options
            .iter()
            .any(|option| option.starts_with("ENABLE_LOAD_EXTENSION")),
        "ENABLE_LOAD_EXTENSION is among {options:?}"
    );
}

#[test]
fn fr_lite_001_the_library_that_answers_is_the_bundled_one() {
    let connection = Connection::open_in_memory().expect("an in-memory database opens");
    let answered: String = connection
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .expect("the version probe answers");
    let bundled = rusqlite::ffi::SQLITE_VERSION
        .to_str()
        .expect("the bundled version is ASCII");

    assert_eq!(answered, bundled);
}
