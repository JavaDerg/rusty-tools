use crate::io::{DataSource, ReadState};
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub const BUFFER_SIZE: usize = 65535;

pub struct FileReader {
    path: PathBuf,
    file: File,
    separator: u32,
    buffer: [u8; BUFFER_SIZE],
    buf_pos: u16,
    buf_size: u16,
    references: Vec<FileRef>,
    ref_pos: u32
}

impl FileReader {
    pub fn new(path: String, cache: bool) -> Result<Self, String> {
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
            path: Default::default(),
            file,
            separator: 13, // New line character,
            buffer: [0u8; BUFFER_SIZE],
            buf_pos: 0,
            buf_size: 0,
            references: Vec::new(),
            ref_pos: 0
        };
        loop {
            fr.references.push(match fr.read_next_line_internal(cache) {
                ReadState::Successful(line) => line,
                ReadState::EndOfData => break,
                ReadState::Error(e) => return Err(e)
            });
        }
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

    pub(in self) fn read_next_line_internal(&mut self, cache: bool) -> ReadState<FileRef> {
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
                        let len = pos - starting_pos;
                        return ReadState::Successful(FileRef {
                            pos: starting_pos,
                            len: len as usize,
                            content: if cache {
                                if let Err(e) = file.seek(SeekFrom::Current(-len as i64)) {
                                    return ReadState::Error(format!("Failed setting stream position: {}", e));
                                }
                                let mut buffer = vec![0u8; self.len];
                                match file.read(buffer.as_mut()) {
                                    Ok(x) => {
                                        if x < len as usize {
                                            return ReadState::Error(format!("Reading the file failed, size of returned bytes was to small. Expected: {} Got: {}", self.len, x));
                                        }
                                        Some(buffer)
                                    }
                                    Err(e) => ReadState::Error(format!("Reading the file failed: {}", e))
                                }
                            } else { None }
                        });
                    }
                    return ReadState::EndOfData;
                }
                ReadState::Error(x) => return ReadState::Error(x)
            };
            let len = (match self.file.seek(SeekFrom::Current(0)) {
                Ok(x) => x,
                Err(e) => return ReadState::Error(format!("Failed to get stream position"))
            } - starting_pos) as usize;
            if c == self.separator {
                return ReadState::Successful(FileRef {
                    pos: starting_pos,
                    len,
                    content: if cache {
                        if let Err(e) = file.seek(SeekFrom::Current(-len as i64)) {
                            return ReadState::Error(format!("Failed setting stream position: {}", e));
                        }
                        let mut buffer = vec![0u8; self.len];
                        match file.read(buffer.as_mut()) {
                            Ok(x) => {
                                if x < len as usize {
                                    return ReadState::Error(format!("Reading the file failed, size of returned bytes was to small. Expected: {} Got: {}", self.len, x));
                                }
                                Some(buffer)
                            }
                            Err(e) => ReadState::Error(format!("Reading the file failed: {}", e))
                        }
                    } else { None }
                });
            }
        }
    }
}

impl DataSource for FileReader {
    fn next_line(&mut self) -> ReadState<Vec<u8>> {
        if self.ref_pos >= self.references.len() as u32 {
            return ReadState::EndOfData;
        }
        let r: &FileRef = unsafe { self.references.get_unchecked(self.ref_pos) };
        self.ref_pos += 1;
        ReadState::Successful(match r.read(self) {
            ReadState::Successful(x) => x,
            ReadState::EndOfData => panic!("Reading a FileRef returned EndOfData. This is a invalid state!"),
            ReadState::Error(e) => return ReadState::Error(e)
        })
    }
}

pub struct FileRef {
    pos: u64,
    len: usize,
    content: Option<Vec<u8>>
}

impl FileRef {
    fn read(&self, fr: &mut FileReader) -> ReadState<Vec<u8>> {
        let file = &mut fr.file;
        let cur_pos = match file.seek(SeekFrom::Current(0)) {
            Ok(x) => x,
            Err(e) => return ReadState::Error(format!("Failed reading stream position: {}", e))
        };
        if let Err(e) = file.seek(SeekFrom::Start(self.pos)) {
            return ReadState::Error(format!("Failed setting stream position: {}", e));
        }
        let mut buffer = vec![0u8; self.len];
        match file.read(buffer.as_mut()) {
            Ok(x) => {
                if x < self.len as usize {
                    return ReadState::Error(format!("Reading the file failed, size of returned bytes was to small. Expected: {} Got: {}", self.len, x));
                }
                ReadState::Successful(buffer)
            }
            Err(e) => ReadState::Error(format!("Reading the file failed: {}", e))
        }
    }
}
