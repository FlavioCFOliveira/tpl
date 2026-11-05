#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! A data layer for three MariaDB schemas, one MySQL schema and one SQLite
//! database, rendered from their catalogue.
//!
//! This crate is the output of the Rust worked example. Everything below this
//! file is generated: one module per schema, and three modules per table of
//! that schema — the row, the repository trait, and the `sqlx` implementation
//! of that trait. Nothing here is written by hand except this file, which
//! carries the crate's attributes and names the five schema modules.
//!
//! | Module | Provider | Schema | Tables |
//! |---|---|---|---|
//! | [`sakila`] | MariaDB | `sakila` | 16 |
//! | [`world`] | MariaDB | `world` | 3 |
//! | [`freight`] | MariaDB | `freight` | 17 |
//! | [`freight_mysql`] | MySQL | `freight` | 22 |
//! | [`freight_sqlite`] | SQLite | `main` | 24 |
//!
//! The five are rendered by one template set, the first three from one server,
//! so what differs between them is the catalogue and nothing else.
//!
//! # Using it
//!
//! ```no_run
//! use sqlx::mysql::MySqlConnectOptions;
//! use tpl_example_rust_data_layer::sakila::{self, ActorColumn, ActorFilter, ActorOrder, ActorRepository, Direction};
//!
//! # async fn example() -> Result<(), sakila::Error> {
//! let pool = sakila::connect(MySqlConnectOptions::new().host("127.0.0.1")).await?;
//! let repositories = sakila::Repositories::new(&pool);
//!
//! let found = repositories
//!     .actor
//!     .search(
//!         ActorFilter { last_name_like: Some("KIL%".to_owned()), ..ActorFilter::default() },
//!         &[ActorOrder { column: ActorColumn::LastName, direction: Direction::Ascending }],
//!         Some(10),
//!         None,
//!     )
//!     .await?;
//! # let _ = found;
//! # Ok(())
//! # }
//! ```
//!
//! `unsafe` is forbidden, in this crate as in the tool that rendered it.

pub mod freight;
pub mod freight_mysql;
pub mod freight_sqlite;
pub mod sakila;
pub mod world;
