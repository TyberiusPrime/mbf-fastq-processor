#[derive(Clone)]
pub struct FastQRead {
    pub name: Vec<u8>,
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
}

impl FastQRead {
    #[must_use]
    pub fn fo_fastq(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(b'@');
        out.extend_from_slice(&self.name);
        out.push(b'\n');
        out.extend_from_slice(&self.seq);
        out.push(b'\n');
        out.push(b'+');
        out.push(b'\n');
        out.extend_from_slice(&self.qual);
        out.push(b'\n');
        out
    }

    #[must_use]
    pub fn cut_start(&self, n: usize) -> Self {
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[n..].to_vec(),
            qual: self.qual[n..].to_vec(),
        }
    }

    #[must_use]
    pub fn cut_end(&self, n: usize) -> Self {
        let remaining = self.seq.len().saturating_sub(n);
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[..remaining].to_vec(),
            qual: self.qual[..remaining].to_vec(),
        }
    }

    #[must_use]
    pub fn max_len(&self, n: usize) -> Self {
        let remaining = self.seq.len().min(n);
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[..remaining].to_vec(),
            qual: self.qual[..remaining].to_vec(),
        }
    }

    #[must_use]
    pub fn prefix(&self, seq: &[u8], qual: &[u8]) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(seq);
        new_seq.extend_from_slice(&self.seq);
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(qual);
        new_qual.extend_from_slice(&self.qual);
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }

    #[must_use]
    pub fn postfix(&self, seq: &[u8], qual: &[u8]) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(&self.seq);
        new_seq.extend_from_slice(seq);
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(&self.qual);
        new_qual.extend_from_slice(qual);
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }

    #[must_use]
    pub fn reverse(&self) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(&self.seq);
        new_seq.reverse();
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(&self.qual);
        new_qual.reverse();
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }
}
