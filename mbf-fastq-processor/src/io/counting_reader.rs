use std::{io::Read, sync::{atomic::AtomicUsize, Arc}};


pub struct CountingReader<T: std::io::Read+Send> {
    inner: T,
    counter: Arc<AtomicUsize>,
}

impl <T: std::io::Read+Send> CountingReader<T> {
    pub fn new(inner: T, counter: Arc<AtomicUsize>) -> CountingReader<T> {
        CountingReader { inner, counter }
    }
}

impl <T: std::io::Read+Send> Read for CountingReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        self.counter.fetch_add(bytes_read, std::sync::atomic::Ordering::Relaxed);
        Ok(bytes_read)
    }
}



