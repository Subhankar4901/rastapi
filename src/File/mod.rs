use std::{
    fs::File,
    io::{BufReader, Read, Result},
    iter,
};
pub struct FileWrapper {
    file_buffer: BufReader<File>,
    chunk_size: usize,
}
impl FileWrapper {
    pub fn new(file: File, chunk_size: Option<usize>) -> Self {
        let chunk_size = chunk_size.unwrap_or(1 << 15);
        let file_buffer = BufReader::new(file);
        Self {
            file_buffer,
             chunk_size,
        }
    }
    pub fn iter(&mut self) -> impl Iterator<Item = Result<Vec<u8>>> + '_ {
        iter::from_fn(move || {
            let mut buffer: Vec<u8> = vec![0; self.chunk_size];
            match self.file_buffer.read(&mut buffer) {
                Ok(0) => None,
                Ok(n) => {
                    buffer.truncate(n);
                    Some(Ok(buffer))
                }
                Err(e) => Some(Err(e.into())),
            }
        })
    }
}
