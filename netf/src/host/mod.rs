use std::net::{IpAddr, TcpListener, ToSocketAddrs};
use std::process::exit;
use std::{thread, io};
use std::io::{Write, Seek, SeekFrom};
use std::fs::File;
use byteorder::{BigEndian, ByteOrder};

pub fn host_file_tcp(file: String, target: String) -> Result<(), String> {
    let mut listener = match TcpListener::bind(target) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("An error occurred opening the tcp listener: {}", e);
            exit(1);
        }
    };
    let mut fail_count = 0u8;
    let mut con_id = 0u64;
    loop {
        let (mut stream, addrs) = match listener.accept() {
            Ok(x) => {
                fail_count = 0;
                x
            },
            Err(e) => {
                eprintln!("Unknown error occurred trying to accept client: {}", e);

                fail_count += 1;
                if fail_count >= 10 {
                    eprintln!("Trying to accept a client failed 10 or more times in a row. \
                    Aborting program");
                    exit(1);
                }
                continue;
            }
        };
        eprintln!("[{}] {}:{} connected", con_id, addrs.ip(), addrs.port());
        let id_c = con_id.clone();
        let file_c = file.clone();
        let mut fs = match File::open(file_c) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                exit(1);
            }
        };
        thread::spawn(move || {
            eprintln!("[{}] Thread created, sending data...", id_c);
            write([0u8].as_ref()); // No encryption, TODO: Implement encryption
            let size = match fs.seek(SeekFrom::Start(0)) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("[{}] Failed to seek file: {}", id_c, e);
                    exit(1);
                }
            };
            {
                let mut s_buf = [0u8; 8];
                BigEndian::write_u64(&mut s_buf, size);
                write(&s_buf);
            }

            fn write(buf: &[u8]) {
                match stream.write(buf) {
                    Ok(x) => {
                        if x != 1 {
                            eprintln!("[{}] {} bytes were send instead of 1", id_c, x);
                            return
                        }
                    },
                    Err(e) => {
                        eprintln!("[{}] An error occurred sending data: {}", id_c, e);
                    }
                }
            }
        });
    }
    Ok(())
}