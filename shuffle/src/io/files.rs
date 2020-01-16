use crate::io::{DataSource, DataReference, ReadState};
use std::path::PathBuf;
use std::cell::RefCell;
use core::mem;
use std::ptr::NonNull;
use std::fs::File;
use std::io::Read;
use crate::io::files::ReadState::EndOfData;

pub const BUFFER_SIZE: usize = 65535;

pub struct FileReader {
    this: NonNull<Self>,
    path: PathBuf,
    file: File,
    pos: u64,
    seperator: u64,
    buffer: [u8; BUFFER_SIZE],
    buf_pos: u16,
    buf_size: u16
}

impl FileReader {
    pub fn new(path: String) -> Result<Self, String> {
        let mut pb = PathBuf::from(path);
        if !pb.exists() {
            return Err("File not found".to_string());
        }
        let mut file = match File::open(pb) {
            Ok(x) => x,
            Err(x) => {
                return Err(format!("Opening file failed: {}", x).to_string());
            }
        };
        let mut fr = FileReader {
            this: unsafe { mem::MaybeUninit::uninit().assume_init() },
            path: Default::default(),
            file,
            pos: 0,
            seperator: 13, // New line character,
            buffer: [u8; BUFFER_SIZE],
            buf_pos: 0,
            buf_size: 0
        };
        fr.this = NonNull::from(&mut fr);
        Ok(fr)
    }

    fn refill_buffer(&mut self) -> ReadState<()> {
        self.pos = 0;
        self.buf_size = match self.file.read(&mut self.buffer) {
            Ok(x) => {
                if x == 0 {
                    return ReadState::EndOfData;
                }
                x
            },
            Err(e) => return ReadState::Error(format!("Reading line from file failed: {}", e))
        } as u16;
        ReadState::Successful(())
    }
}

impl DataSource for FileReader {
    fn next_line(&mut self) -> Result<Option<Box<dyn DataReference>>, String> {
        if self.buf_size == 0 {

        }

        ()
    }

    fn pos(&self) -> u64 {
        self.pos
    }

    fn shift(&mut self, n: i64) -> Result<(), ()> {
        unimplemented!()
    }
}

pub struct FileRef {
    owner: NonNull<FileReader>, // This is a dangling pointer!!!
    pos: u64,
    len: u64 /*, TODO: Implement this lol
    content: Option<Box<[u8]>> */
}

impl DataReference for FileRef {
    fn read(&mut self) -> &[u8] {
        unimplemented!()
    }
}
