use rusqlite::Connection;

pub fn setup_schema(db: Connection) -> Result<(), anyhow::Error> {
    if let Err(e) = db.execute(
        "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            data  BLOB
        )",
        (), // empty list of parameters.
    ) {
        eprintln!("Error: {}", e);
    } else {
        println!("Table Created: person");
    }
    Ok(())
}
