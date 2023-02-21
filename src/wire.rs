//! Chat application via custom wire protocol over TCP.
//!
//! The wire protocol used is just a single byte indicating the type of message
//! being sent, followed by the payload itself. All variable-length parts of the
//! payload have a length prefixed. If the length is less than 255 bytes, then
//! it's just encoded as a single byte. Otherwise it starts with a byte of value
//! 0, followed by the length encoded in 4 bytes (big endian).
//!
//! I'd probably use `bincode` for this in a real application, but for
//! pedagogical reasons this exercise forbids other libraries.
//!
//! Run this program with `cargo run wire [client|server]`.

use std::{
    collections::{btree_map::Entry, BTreeMap},
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use colored::Colorize;
use parking_lot::Mutex;
use wildmatch::WildMatch;

/// Arbitrary local port for client and server communications.
pub const WIRE_PORT: u16 = 5722;

/// A unified message type for client and server.
pub enum Message {
    /// Create an account.
    Create(String),

    /// List accounts, optionally by text wildcard.
    List(String),

    /// Send message to a recipient.
    Send(String, String),

    /// Deliver undelivered messages to a particular user.
    Deliver(String),

    /// Delete an account (fails if it has queued messages).
    Delete(String),

    /// Returned by the server.
    Response(Result<String, String>),
}

impl Message {
    fn encode_len(stream: &mut impl Write, len: usize) -> io::Result<()> {
        let len = u32::try_from(len).expect("message too long");
        if len < 255 {
            stream.write_all(&[len as u8])
        } else {
            let buf = len.to_be_bytes();
            stream.write_all(&[255, buf[0], buf[1], buf[2], buf[3]])
        }
    }

    fn encode_str(stream: &mut impl Write, s: &str) -> io::Result<()> {
        Self::encode_len(stream, s.len())?;
        stream.write_all(s.as_bytes())
    }

    fn decode_len(stream: &mut impl Read) -> io::Result<usize> {
        let mut buf = [0; 4];
        stream.read_exact(&mut buf[..1])?;
        let len = if buf[0] == 255 {
            stream.read_exact(&mut buf)?;
            u32::from_be_bytes(buf) as usize
        } else {
            buf[0] as usize
        };
        Ok(len)
    }

    fn decode_str(stream: &mut impl Read) -> io::Result<String> {
        let len = Self::decode_len(stream)?;
        let mut buf = vec![0; len];
        stream.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "wire message had invalid UTF-8")
        })
    }

    /// Encode a message onto a writable stream.
    pub fn encode(&self, stream: &mut impl Write) -> io::Result<()> {
        match self {
            Message::Create(name) => {
                stream.write_all(&[1])?;
                Self::encode_str(stream, name)
            }
            Message::List(filter) => {
                stream.write_all(&[2])?;
                Self::encode_str(stream, filter)
            }
            Message::Send(name, text) => {
                stream.write_all(&[3])?;
                Self::encode_str(stream, name)?;
                Self::encode_str(stream, text)
            }
            Message::Deliver(name) => {
                stream.write_all(&[4])?;
                Self::encode_str(stream, name)
            }
            Message::Delete(name) => {
                stream.write_all(&[5])?;
                Self::encode_str(stream, name)
            }
            Message::Response(Ok(resp)) => {
                stream.write_all(&[242])?;
                Self::encode_str(stream, resp)
            }
            Message::Response(Err(err)) => {
                stream.write_all(&[243])?;
                Self::encode_str(stream, err)
            }
        }
    }

    /// Decode the next message from a readable stream.
    pub fn decode(stream: &mut impl Read) -> io::Result<Self> {
        let mut buf = [0];
        stream.read_exact(&mut buf)?;
        match buf[0] {
            1 => Ok(Message::Create(Self::decode_str(stream)?)),
            2 => Ok(Message::List(Self::decode_str(stream)?)),
            3 => Ok(Message::Send(
                Self::decode_str(stream)?,
                Self::decode_str(stream)?,
            )),
            4 => Ok(Message::Deliver(Self::decode_str(stream)?)),
            5 => Ok(Message::Delete(Self::decode_str(stream)?)),
            242 => Ok(Message::Response(Ok(Self::decode_str(stream)?))),
            243 => Ok(Message::Response(Err(Self::decode_str(stream)?))),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "wire message had invalid type",
            )),
        }
    }
}

pub fn run_client() -> io::Result<()> {
    let mut stream = TcpStream::connect(("127.0.0.1", WIRE_PORT))?;

    // This was also mostly written by Copilot.
    loop {
        let mut line = String::new();
        eprint!("{}", "wire> ".green());
        io::stdin().read_line(&mut line)?;
        let mut words = line.split_whitespace();
        let Some(cmd) = words.next() else { break };
        match cmd {
            "create" => {
                let Some(name) = words.next() else {
                    eprintln!("missing argument");
                    continue;
                };
                Message::Create(name.into()).encode(&mut stream)?;
            }
            "list" => {
                let filter = words.next().unwrap_or("");
                Message::List(filter.into()).encode(&mut stream)?;
            }
            "send" => {
                let Some(name) = words.next() else {
                    eprintln!("missing argument");
                    continue;
                };
                let text = words.collect::<Vec<_>>().join(" ");
                Message::Send(name.into(), text).encode(&mut stream)?;
            }
            "deliver" => {
                let Some(name) = words.next() else {
                    eprintln!("missing argument");
                    continue;
                };
                Message::Deliver(name.into()).encode(&mut stream)?;
            }
            "delete" => {
                let Some(name) = words.next() else {
                    eprintln!("missing argument");
                    continue;
                };
                Message::Delete(name.into()).encode(&mut stream)?;
            }
            _ => {
                eprintln!("unknown command");
                continue;
            }
        }

        match Message::decode(&mut stream)? {
            Message::Response(Ok(resp)) => print!("{}", resp.yellow()),
            Message::Response(Err(err)) => eprintln!("{} {}", "error:".red(), err),
            _ => eprintln!("unexpected response"),
        }
    }

    Ok(())
}

pub fn run_server() -> io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", WIRE_PORT))?;

    // All state for the server is in this threadsafe map.
    let accounts: Arc<Mutex<BTreeMap<String, Vec<String>>>> = Default::default();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("error accepting connection: {}", err);
                continue;
            }
        };

        let accounts = Arc::clone(&accounts);

        thread::spawn::<_, io::Result<()>>(move || loop {
            // Most of this part was written by Copilot.
            let resp = match Message::decode(&mut stream)? {
                Message::Create(name) => {
                    eprintln!("create account {}", name);
                    let mut accounts = accounts.lock();
                    if accounts.contains_key(&name) {
                        Err("account already exists".into())
                    } else {
                        accounts.insert(name.clone(), Vec::new());
                        Ok("".into())
                    }
                }
                Message::List(filter) => {
                    let matcher = if filter.is_empty() {
                        WildMatch::new("*")
                    } else {
                        WildMatch::new(&filter)
                    };

                    let mut results = String::new();
                    let accounts = accounts.lock();
                    for key in accounts.keys() {
                        if matcher.matches(key) {
                            results += key;
                            results += "\n";
                        }
                    }
                    Ok(results)
                }
                Message::Send(name, text) => {
                    eprintln!("send message to {}", name);
                    let mut accounts = accounts.lock();
                    if let Some(queue) = accounts.get_mut(&name) {
                        queue.push(text.clone());
                        Ok("".into())
                    } else {
                        Err("account does not exist".into())
                    }
                }
                Message::Deliver(name) => {
                    eprintln!("deliver messages to {}", name);
                    let mut accounts = accounts.lock();
                    if let Some(queue) = accounts.get_mut(&name) {
                        let mut results = String::new();
                        for msg in queue.drain(..) {
                            results += &msg;
                            results += "\n";
                        }
                        Ok(results)
                    } else {
                        Err("account does not exist".into())
                    }
                }
                Message::Delete(name) => {
                    eprintln!("delete account {}", name);
                    let mut accounts = accounts.lock();
                    match accounts.entry(name) {
                        Entry::Occupied(entry) => {
                            if entry.get().is_empty() {
                                entry.remove();
                                Ok("".into())
                            } else {
                                Err("account has messages".into())
                            }
                        }
                        Entry::Vacant(_) => Err("account does not exist".into()),
                    }
                }
                _ => {
                    eprintln!("unexpected message from client");
                    continue;
                }
            };
            Message::Response(resp).encode(&mut stream)?;
        });
    }

    Ok(())
}
