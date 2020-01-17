mod files;

pub trait DataSource {
    fn next_line(&mut self) -> ReadState<Box<dyn DataReference>>;
}

pub trait DataReference {
    fn read(&mut self) -> Result<Vec<u8>, String>;
}

enum ReadState<T> {
    Successful(T),
    EndOfData,
    Error(String)
}
