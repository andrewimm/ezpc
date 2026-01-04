//! GDB socket thread - handles non-blocking I/O
//!
//! Runs in a separate thread to avoid blocking the emulation loop.
//! Reads packets from socket → incoming queue
//! Writes packets from outgoing queue → socket

use super::protocol::{parse_packet, ACK};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Spawn socket listener thread
///
/// Returns thread handle. Thread runs until incoming queue is dropped.
pub fn spawn_socket_listener(
    socket_path: String,
    incoming: Arc<RwLock<VecDeque<String>>>,
    outgoing: Arc<RwLock<VecDeque<String>>>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        // Remove old socket file if it exists
        let _ = std::fs::remove_file(&socket_path);

        // Bind Unix socket
        let listener = match UnixListener::bind(&socket_path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind GDB socket at {}: {}", socket_path, e);
                return;
            }
        };

        println!("GDB server listening on {}", socket_path);

        // Accept connections (one at a time)
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    println!("GDB client connected");

                    // Handle this connection
                    if let Err(e) = handle_connection(stream, incoming.clone(), outgoing.clone()) {
                        eprintln!("GDB connection error: {}", e);
                    }

                    println!("GDB client disconnected");
                }
                Err(e) => {
                    eprintln!("GDB accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    })
}

/// Handle a single GDB connection
fn handle_connection(
    mut stream: UnixStream,
    incoming: Arc<RwLock<VecDeque<String>>>,
    outgoing: Arc<RwLock<VecDeque<String>>>,
) -> std::io::Result<()> {
    // Set non-blocking mode
    stream.set_nonblocking(true)?;

    let mut read_buffer = Vec::new();
    let mut write_buffer = Vec::new();

    loop {
        // Read from socket → incoming queue
        let mut temp_buf = [0u8; 4096];
        match stream.read(&mut temp_buf) {
            Ok(0) => {
                // Connection closed
                return Ok(());
            }
            Ok(n) => {
                // Append to read buffer
                read_buffer.extend_from_slice(&temp_buf[..n]);

                // Try to parse packets
                while let Some(packet) = parse_packet(&read_buffer) {
                    // Send ACK
                    stream.write_all(ACK)?;

                    // Push packet to incoming queue
                    incoming.write().unwrap().push_back(packet.clone());

                    // Remove parsed packet from buffer
                    // Find end of packet (# + 2 checksum digits)
                    if let Some(hash_pos) = read_buffer.iter().position(|&b| b == b'#') {
                        if read_buffer.len() >= hash_pos + 3 {
                            read_buffer.drain(..hash_pos + 3);
                        }
                    }
                }

                // Also handle ACK/NAK bytes (just discard them for now)
                while !read_buffer.is_empty() && (read_buffer[0] == b'+' || read_buffer[0] == b'-')
                {
                    read_buffer.remove(0);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available, continue
            }
            Err(e) => {
                return Err(e);
            }
        }

        // Outgoing queue → socket
        {
            let mut queue = outgoing.write().unwrap();
            while let Some(response) = queue.pop_front() {
                write_buffer.extend_from_slice(response.as_bytes());
            }
        }

        // Write buffered data to socket
        if !write_buffer.is_empty() {
            match stream.write(&write_buffer) {
                Ok(n) => {
                    write_buffer.drain(..n);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Can't write now, try again later
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // Small sleep to avoid busy loop
        thread::sleep(Duration::from_micros(100));
    }
}
