use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

pub const SERIAL_DATA_ADDR: u16 = 0xBFE0; // read: rx byte; write: tx byte
pub const SERIAL_STATUS_ADDR: u16 = 0xBFE1; // read: bit 7 set (0x80) if rx byte waiting

pub struct Serial {
    rx_receiver: Receiver<u8>,
    tx_sender: Sender<u8>,
    buffer: Mutex<Option<u8>>,
}

impl Serial {
    pub fn new(port: u16) -> Self {
        let (tx_out, rx_out) = channel::<u8>(); // cpu -> term
        let (tx_in, rx_in) = channel::<u8>();   // term -> cpu

        thread::spawn(move || {
            let addr = format!("127.0.0.1:{}", port); // open new (host) CPU thread for the serial monitor
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(_) => return,
            };

            for mut stream in listener.incoming().flatten() {
                stream.set_nonblocking(true).ok();

                let tx_in = tx_in.clone();
                let mut buf = [0u8; 128];

                loop {
                    // read from externam terminal -> send to cpu
                    if let Ok(count) = stream.read(&mut buf) {
                        if count == 0 {
                            break;
                        }

                        for &byte in &buf[..count] {
                            let mut char_byte = if byte == b'\n' {
                                b'\r'
                            } else {
                                byte
                            };
                            char_byte = char_byte.to_ascii_uppercase();
                            tx_in.send(char_byte).ok();
                        }
                    }

                    // read from cpu -> send to external terminal
                    while let Ok(byte) = rx_out.try_recv() {
                        if byte == b'\r' {
                            stream.write_all(b"\r\n").ok();
                        } else {
                            stream.write_all(&[byte]).ok();
                        }
                        stream.flush().ok();
                    }

                    thread::sleep(std::time::Duration::from_millis(1));
                }
            }
        });

        Serial {
            rx_receiver: rx_in,
            tx_sender: tx_out,
            buffer: Mutex::new(None),
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            SERIAL_STATUS_ADDR => {
                let mut guard = self.buffer.lock().unwrap();
                if guard.is_none() {
                    *guard = self.rx_receiver.try_recv().ok();
                }
                if guard.is_some() {
                    0x80
                } else {
                    0x00
                }
            },
            SERIAL_DATA_ADDR => {
                let mut guard = self.buffer.lock().unwrap();
                if guard.is_none() {
                    *guard = self.rx_receiver.try_recv().ok();
                }
                guard.take().unwrap_or(0)
            },
            _ => 0,
        }
    }

    pub fn write_register(&mut self, addr: u16, data: u8) {
        if addr == SERIAL_DATA_ADDR {
            self.tx_sender.send(data).ok();
        }
    }
}
