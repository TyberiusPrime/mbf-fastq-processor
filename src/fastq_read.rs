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

    #[must_use]
    pub fn reverse_complement(&self) -> Self {
        let new_seq = reverse_complement_iupac(&self.seq);
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

pub fn reverse_complement_iupac(input: &[u8]) -> Vec<u8> {
    let mut new_seq = Vec::new();
    for char in input.iter().rev() {
        new_seq.push(match char {
            b'A' => b'T',
            b'T' => b'A',
            b'C' => b'G',
            b'G' => b'C',
            b'U' => b'A',

            b'a' => b't',
            b't' => b'a',
            b'c' => b'g',
            b'g' => b'c',
            b'u' => b'a',

            b'R' => b'Y',
            b'Y' => b'R',
            b'S' => b'S',
            b'W' => b'W',
            b'K' => b'M',
            b'M' => b'K',
            b'B' => b'V',
            b'V' => b'B',
            b'D' => b'H',
            b'H' => b'D',

            b'r' => b'y',
            b'y' => b'r',
            b's' => b's',
            b'w' => b'w',
            b'k' => b'm',
            b'm' => b'k',
            b'b' => b'v',
            b'v' => b'b',
            b'd' => b'h',
            b'h' => b'd',
            b'\n' => panic!("New line in DNA sequence"),
            _ => *char,
        });
    }
    new_seq
}
#[cfg(test)]
mod test {

    fn check(should: &[u8], input: &[u8]) {
        let s: Vec<u8> = should.iter().map(|x| *x).collect();
        assert_eq!(s, super::reverse_complement_iupac(input));
    }
    #[test]
    fn test_rev_complement() {
        check(b"AGCT", b"AGCT");
        check(b"DHBVNKMWSRYAAGCT", b"AGCTURYSWKMNBVDH");
    }
}
