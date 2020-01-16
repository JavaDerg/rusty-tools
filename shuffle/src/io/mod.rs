mod files;

pub trait DataSource {
    fn next_line(&mut self) -> Option<Box<dyn DataReference>>;
    fn pos(&self) -> u64;
    fn shift(&mut self, n: i64) -> Result<(), ()>;
}

pub trait DataReference {
    fn read(&mut self) -> &[u8];
}

enum ReadState<T> {
    Successful(T),
    EndOfData,
    Error(String)
}
