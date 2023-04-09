//! Replicated chat server based on [`wire`].
//!
//! The server here differs in using a persistent SQLite database to store the
//! messages, rather than an in-memory data structure. It also can bind to the
//! same address multiple times, for fault-tolerance.

use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    thread,
};

use rusqlite::{Connection, OptionalExtension};
use socket2::{Domain, Socket, Type};
use wildmatch::WildMatch;

use crate::wire::{self, Message, WIRE_PORT};

pub const DATABASE_FILE: &str = "chat.sqlite";

fn db_connect() -> rusqlite::Result<Connection> {
    let conn = Connection::open(DATABASE_FILE)?;
    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    Ok(conn)
}

fn db_initialize() -> rusqlite::Result<()> {
    let conn = db_connect()?;
    conn.execute_batch(
        "BEGIN;
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
            message TEXT NOT NULL
        );
        COMMIT;",
    )?;
    Ok(())
}

struct HandleError(String);

impl<T: ToString> From<T> for HandleError {
    fn from(err: T) -> Self {
        HandleError(err.to_string())
    }
}

fn handle_message(conn: &mut Connection, message: Message) -> Result<String, HandleError> {
    match message {
        Message::Create(name) => {
            eprintln!("create account {name}");
            let mut stmt = conn.prepare_cached("INSERT INTO users (name) VALUES (?)")?;
            stmt.execute([&name])?;
            Ok("".into())
        }
        Message::List(filter) => {
            let matcher = if filter.is_empty() {
                WildMatch::new("*")
            } else {
                WildMatch::new(&filter)
            };

            let mut stmt = conn.prepare_cached("SELECT name FROM users")?;
            let names = match stmt.query_map([], |row| row.get(0)) {
                Ok(rows) => rows.collect::<Result<Vec<String>, _>>()?,
                Err(err) => {
                    let str = err.to_string();
                    if str.contains("UNIQUE constraint failed: users.name") {
                        return Err("account already exists".into());
                    } else {
                        return Err(str.into());
                    }
                }
            };
            let mut results = String::new();
            for name in names {
                if matcher.matches(&name) {
                    results += &name;
                    results += "\n";
                }
            }
            Ok(results)
        }
        Message::Send(name, text) => {
            eprintln!("send message to {name}");
            let mut stmt = conn.prepare_cached(
                "INSERT INTO messages (user_id, message)
                VALUES ((SELECT id FROM users WHERE name = ?), ?)",
            )?;
            match stmt.execute([&name, &text]) {
                Ok(_) => Ok("".into()),
                Err(err) => {
                    let str = err.to_string();
                    if str.contains("NOT NULL constraint failed: messages.user_id") {
                        Err("account does not exist".into())
                    } else {
                        Err(str.into())
                    }
                }
            }
        }
        Message::Deliver(name) => {
            eprintln!("deliver messages to {name}");
            let txn = conn.transaction()?;
            let mut results = String::new();
            {
                let mut stmt = txn.prepare_cached("SELECT id FROM users WHERE name = ?")?;
                let Some(user_id) = stmt.query_row([&name], |row| row.get::<_, u64>(0)).optional()? else {
                    return Err("account does not exist".into());
                };
                let mut stmt = txn
                    .prepare_cached("DELETE FROM messages WHERE user_id = ? RETURNING message")
                    .unwrap();
                for message_result in stmt.query_map([&user_id], |row| row.get(0))? {
                    let message: String = message_result?;
                    results += &message;
                    results += "\n";
                }
            }
            txn.commit()?;
            Ok(results)
        }
        Message::Delete(name) => {
            eprintln!("delete account {name}");
            let mut stmt = conn.prepare_cached("DELETE FROM users WHERE name = ?")?;
            match stmt.execute([&name]) {
                Ok(0) => Err("account does not exist".into()),
                Ok(_) => Ok("".into()),
                Err(err) => {
                    let str = err.to_string();
                    if str.contains("FOREIGN KEY constraint failed") {
                        Err("account has messages".into())
                    } else {
                        Err(str.into())
                    }
                }
            }
        }
        _ => {
            eprintln!("unexpected message from client");
            Ok("".into())
        }
    }
}

pub fn run_client() {
    // The application client remains the same as before.
    wire::run_client()
}

pub fn run_server() -> anyhow::Result<()> {
    // Connect to the database and initialize tables.
    db_initialize()?;

    // Set initial socket options to allow reuse of port.
    let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddrV4::new(Ipv4Addr::LOCALHOST, WIRE_PORT).into())?;
    socket.listen(128)?;

    let listener = TcpListener::from(socket);

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("error accepting connection: {}", err);
                continue;
            }
        };

        let mut conn = db_connect()?;
        thread::spawn(move || loop {
            let Ok(message) = Message::decode(&mut stream) else { break };
            let resp = handle_message(&mut conn, message).map_err(|err| err.0);
            let Ok(_) = Message::Response(resp).encode(&mut stream) else { break };
        });
    }

    Ok(())
}
