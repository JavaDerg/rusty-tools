pub mod files;

pub trait DataSource {
    fn next_line(&mut self) -> ReadState<Vec<u8>>;
}

pub enum ReadState<T> {
    Successful(T),
    EndOfData,
    Error(String)
}
