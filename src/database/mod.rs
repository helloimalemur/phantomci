use rusqlite::Connection;

pub fn setup_schema(db: Connection) -> Result<(), anyhow::Error> {
    if let Err(e) = db.execute(
        "CREATE TABLE person (
            id    INTEGER PRIMARY KEY,
            sha  TEXT NOT NULL,
            date  BLOB
        )",
        (),
    ) {
        eprintln!("Error: {}", e);
    } else {
        println!("Table Created: person");
    }
    
    Ok(())
}
