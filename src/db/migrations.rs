use crate::error::GreedError;
use duckdb::Connection;

struct Migration {
    version: i32,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    // Future schema changes land here, e.g.:
    // Migration { version: 1, sql: "CREATE TABLE portfolio_snapshots (...)" },
];

pub fn apply(connection: &Connection) -> Result<(), GreedError> {
    apply_with(connection, MIGRATIONS)
}

fn apply_with(connection: &Connection, migrations: &[Migration]) -> Result<(), GreedError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
    )?;
    let current_version: i32 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    for migration in migrations.iter().filter(|m| m.version > current_version) {
        connection.execute_batch(migration.sql)?;
        connection.execute(
            "INSERT INTO schema_migrations (version) VALUES (?)",
            [migration.version],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::db::migrations::{apply, apply_with, Migration};
    use duckdb::Connection;

    #[test]
    fn apply_creates_schema_migrations_table_with_no_migrations() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("apply should succeed");

        let version: i32 = connection
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .expect("schema_migrations should be queryable");
        assert_eq!(version, 0);
    }

    #[test]
    fn apply_is_idempotent() {
        let connection = Connection::open_in_memory().expect("in-memory connection");

        apply(&connection).expect("first apply should succeed");
        apply(&connection).expect("second apply should succeed");
    }

    #[test]
    fn apply_with_applies_migrations_in_order() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        let migrations = [
            Migration {
                version: 1,
                sql: "CREATE TABLE foo (id INTEGER)",
            },
            Migration {
                version: 2,
                sql: "CREATE TABLE bar (id INTEGER)",
            },
        ];

        apply_with(&connection, &migrations).expect("apply_with should succeed");

        connection
            .execute_batch("INSERT INTO foo VALUES (1); INSERT INTO bar VALUES (2);")
            .expect("both tables should exist");
    }

    #[test]
    fn apply_with_does_not_reapply_known_migrations() {
        let connection = Connection::open_in_memory().expect("in-memory connection");
        let migrations = [Migration {
            version: 1,
            sql: "CREATE TABLE foo (id INTEGER)",
        }];

        apply_with(&connection, &migrations).expect("first apply_with should succeed");
        // If this re-ran the migration, "CREATE TABLE foo" would fail since it already exists.
        apply_with(&connection, &migrations).expect("second apply_with should succeed");
    }
}
