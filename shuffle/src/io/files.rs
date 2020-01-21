use crate::io::{DataSource, DataReference, ReadState};
use std::path::PathBuf;
use std::cell::RefCell;
use core::mem;
use std::ptr::NonNull;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use crate::io::files::ReadState::EndOfData;

pub const BUFFER_SIZE: usize = 65535;

pub struct FileReader {
    this: NonNull<Self>,
    path: PathBuf,
    file: File,
    seperator: u32,
    buffer: [u8; BUFFER_SIZE],
    buf_pos: u16,
    buf_size: u16,
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
            seperator: 13, // New line character,
            buffer: [0u8; BUFFER_SIZE],
            buf_pos: 0,
            buf_size: 0,
        };
        fr.this = NonNull::from(&mut fr);
        Ok(fr)
    }

    fn refill_buffer(&mut self) -> ReadState<()> {
        self.buf_size = match self.file.read(&mut self.buffer) {
            Ok(x) => {
                if x == 0 {
                    return ReadState::EndOfData;
                }
                x
            }
            Err(e) => return ReadState::Error(format!("Reading line from file failed: {}", e))
        } as u16;
        ReadState::Successful(())
    }

    fn next_byte(&mut self) -> ReadState<u8> {
        if self.buf_pos >= self.buf_size {
            match self.refill_buffer() {
                ReadState::Successful(_) => {}
                ReadState::EndOfData => return ReadState::EndOfData,
                ReadState::Error(x) => return ReadState::Error(x)
            }
        }
        let ret = unsafe { *self.buffer.get_unchecked(self.buf_pos as usize) };
        self.buf_pos += 1;
        ReadState::Successful(ret)
    }

    fn next_utf8_char(&mut self) -> ReadState<u32> {
        let b = match self.next_byte() {
            ReadState::Successful(x) => x,
            ReadState::EndOfData => return ReadState::EndOfData,
            ReadState::Error(x) => return ReadState::Error(x)
        };

        if b >> 7 != 1 {
            return ReadState::Successful(b as u32);
        }
        let mut out = 0u32;
        let mut read = 0u8;
        if b >> 5 == 6 { // 2 bytes inc. this
            out |= (b as u32 & 0x11111) << 6;
            read = 1;
        } else if b >> 4 == 14 { // 3 bytes inc. this
            out |= (b as u32 & 0x1111) << 12;
            read = 2;
        } else if b >> 3 == 30 { // 4 bytes inc. this
            out |= (b as u32 & 0x111) << 18;
            read = 3;
        }

        for i in 0..read {
            let nb = match self.next_byte() {
                ReadState::Successful(x) => x,
                ReadState::EndOfData => return ReadState::EndOfData,
                ReadState::Error(x) => return ReadState::Error(x)
            };
            if nb >> 6 != 2 {
                return ReadState::Successful(0xFFFD); // Encoding this results in [EF, BF, BD] also known as �
            }
            out |= (nb as u32 & 0b111111) << (6 * (2 - i)) as u32;
        }

        ReadState::Successful(out)
    }
}

impl DataSource for FileReader {
    fn next_line(&mut self) -> ReadState<Box<dyn DataReference>> {
        let starting_pos = match self.file.seek(SeekFrom::Current(0)) {
            Ok(x) => x,
            Err(e) => return ReadState::Error(format!("Failed to get stream position"))
        };
        loop {
            let c = match self.next_utf8_char() {
                ReadState::Successful(x) => x,
                ReadState::EndOfData => {
                    let pos = match self.file.seek(SeekFrom::Current(0)) {
                        Ok(x) => x,
                        Err(e) => return ReadState::Error(format!("Failed to get stream position"))
                    };
                    if starting_pos < pos {
                        return ReadState::Successful(Box::new(FileRef {
                            owner: NonNull::from(self),
                            pos: starting_pos,
                            len: pos - starting_pos,
                        }));
                    }
                    return ReadState::EndOfData;
                }
                ReadState::Error(x) => return ReadState::Error(x)
            };
            if c == self.seperator {
                return ReadState::Successful(Box::new(FileRef {
                    owner: NonNull::from(self),
                    pos: starting_pos,
                    len: match self.file.seek(SeekFrom::Current(0)) {
                        Ok(x) => x,
                        Err(e) => return ReadState::Error(format!("Failed to get stream position"))
                    } - starting_pos,
                }));
            }
        }
    }
}

pub struct FileRef {
    // This is a dangling pointer!!! (i think welp)
    owner: NonNull<FileReader>,
    pos: u64,
    len: u64,
    /*, TODO: Implement this lol
       content: Option<Box<[u8]>> */
}

impl DataReference for FileRef {
    fn read(&mut self) -> Result<Box<[u8]>, String> {
        let mut file = &unsafe { self.owner.as_mut() }.file;
        let cur_pos = match file.seek(SeekFrom::Current(0)) {
            Ok(x) => x,
            Err(e) => return Err(format!("Failed reading stream position: {}", e))
        };
        if Err(e) = file.seek(SeekFrom::Start(self.pos)) {
            return Err(format!("Failed setting stream position: {}", e));
        }
        let mut buffer = [0u8, self.len];
        match file.read(&buffer) {
            Ok(x) => {
                if x < self.len as usize {
                    return Err(format!("Reading the file failed, size of returned bytes was to small. Expected: {} Got: {}", self.len, x));
                }
                Ok(Box::new(*buffer))
            },
            Err(e) => Err(format!("Reading the file failed: {}", e))
        }
    }
}
