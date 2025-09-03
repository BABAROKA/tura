use rusqlite::Connection;

pub struct Database {
    pub path: String,
    pub connection: Connection,
}

impl Database {
    pub fn from(path: &str) -> Result<Database, ()> {
        let conn = Connection::open(path).map_err(|err| {
            eprintln!("Unable to establish connection to database {err}");
        })?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS music (
                id INTIGER PRIMARY KEY,
                title TEXT NOT NULL,
                location TEXT NOT NULL
            )",
            (),
        )
        .map_err(|err| {
            eprintln!("Unable to create table music {err}");
        })?;
        Ok(Database {
            path: path.to_string(),
            connection: conn,
        })
    }
    pub fn write() {}
    pub fn read() {}
}
