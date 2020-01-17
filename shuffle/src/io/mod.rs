mod files;

pub trait DataSource {
    fn next_line(&mut self) -> ReadState<Box<dyn DataReference>>;x
}

pub trait DataReference {
    fn read(&mut self) -> Result<&[u8], String>;
}

enum ReadState<T> {
    Successful(T),
    EndOfData,
    Error(String)
}
