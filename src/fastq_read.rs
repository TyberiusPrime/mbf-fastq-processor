#[derive(Clone)]
pub struct FastQRead {
    pub name: Vec<u8>,
    pub seq: Vec<u8>,
    pub qual: Vec<u8>,
}

impl FastQRead {
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

    pub fn cut_start(&self, n: usize) -> Self {
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[n..].to_vec(),
            qual: self.qual[n..].to_vec(),
        }
    }

    pub fn cut_end(&self, n: usize) -> Self {
        let remaining = (self.seq.len() as isize - n as isize).max(0) as usize;
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[..remaining].to_vec(),
            qual: self.qual[..remaining].to_vec(),
        }
    }

    pub fn max_len(&self, n: usize) -> Self {
        let remaining = self.seq.len().min(n);
        FastQRead {
            name: self.name.clone(),
            seq: self.seq[..remaining].to_vec(),
            qual: self.qual[..remaining].to_vec(),
        }
    }

    pub fn prefix(&self, seq: &[u8], qual: &Vec<u8>) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(&seq);
        new_seq.extend_from_slice(&self.seq);
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(&qual);
        new_qual.extend_from_slice(&self.qual);
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }

    pub fn postfix(&self, seq: &[u8], qual: &Vec<u8>) -> Self {
        let mut new_seq = Vec::new();
        new_seq.extend_from_slice(&self.seq);
        new_seq.extend_from_slice(&seq);
        let mut new_qual = Vec::new();
        new_qual.extend_from_slice(&self.qual);
        new_qual.extend_from_slice(&qual);
        FastQRead {
            name: self.name.clone(),
            seq: new_seq,
            qual: new_qual,
        }
    }

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

    pub fn trim_poly_base(
        &self,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_mismatches: usize,
        base: u8,
    ) -> Self {
        fn calc_run_length(
            seq: &[u8],
            query: u8,
            min_length: usize,
            max_mismatch_fraction: f32,
            max_mismatches: usize,
        ) -> Option<usize> {
            if seq.len() < min_length {
                return None;
            }
            /*
            let mut mismatch = 0;
            let mut last_base_pos = seq.len() - 1;
            let mut run_length = 0;
            for ii in 0..seq.len() {
                run_length = ii;
                if seq[seq.len() - ii - 1] != query {
                    mismatch += 1;
                } else {
                    last_base_pos = seq.len() - ii - 1;
                }
                if mismatch > max_mismatches
                    || mismatch as f32 / (ii + 1) as f32 > max_mismatch_fraction 
                    //that's wrong... there might be a longer run than that
                {
                    break;
                }
            }
            if run_length >= min_length {
                return Some(last_base_pos);
            } else {
                return None;
            }
            */
            let mut mm_rate_last_base_pos = Vec::new();
            let mut mismatch_count = 0;
            let mut last_base_pos = seq.len();
            for (ii, c) in seq.iter().rev().enumerate() {
                if *c != query {
                    mismatch_count += 1;
                } else {
                    last_base_pos = seq.len() - ii - 1;
                }
                mm_rate_last_base_pos.push((mismatch_count as f32 / (ii + 1) as f32, last_base_pos, *c));
            }
            dbg!(&mm_rate_last_base_pos);
            //now we take the left most that is below the threshold
            for (ii, (mm_rate, last_base_pos, c)) in mm_rate_last_base_pos.iter().skip(min_length).rev().enumerate() {
                dbg!(ii, *mm_rate, max_mismatch_fraction,c);
                if *mm_rate <= max_mismatch_fraction {
                    dbg!("Hit");
                    return Some(*last_base_pos);
                }
            }
            dbg!("Miss");
            None
            //
        }

        let last_pos = if base == b'.' {
            let lp_a = calc_run_length(&self.seq, b'A', min_length, max_mismatch_fraction, max_mismatches);
            let lp_g = calc_run_length(&self.seq, b'G', min_length, max_mismatch_fraction, max_mismatches);
            let lp_c = calc_run_length(&self.seq, b'C', min_length, max_mismatch_fraction, max_mismatches);
            let lp_t = calc_run_length(&self.seq, b'T', min_length, max_mismatch_fraction, max_mismatches);
            let lp_n = calc_run_length(&self.seq, b'N', min_length, max_mismatch_fraction, max_mismatches);
            //now I need to find the right most one that is not None
            let mut lp = lp_a;
            if lp_g.is_some() && (lp.is_none() || lp_g.unwrap() > lp.unwrap()) {
                lp = lp_g;
            }
            if lp_c.is_some() && (lp_c.is_none() || lp_c.unwrap() > lp.unwrap()) {
                lp = lp_c;
            }
            if lp_t.is_some() && (lp.is_none() || lp_t.unwrap() > lp.unwrap()) {
                lp = lp_t;
            }
            if lp_n.is_some() && (lp.is_none() || lp_n.unwrap() > lp.unwrap()) {
                lp = lp_n;
            }
            lp

        } else {
            calc_run_length(&self.seq, base, min_length, max_mismatch_fraction, max_mismatches)
        };
        if let Some(last_pos) = last_pos  {
            let mut new_seq = Vec::new();
            new_seq.extend_from_slice(&self.seq[..last_pos]);
            let mut new_qual = Vec::new();
            new_qual.extend_from_slice(&self.qual[..last_pos]);
            FastQRead {
                name: self.name.clone(),
                seq: new_seq,
                qual: new_qual,
            }
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_trimm_poly_n() {
        fn trim(seq: &str, min_length: usize, max_mismatch_fraction: f32, base: u8) -> String {
            let read = super::FastQRead {
                name: b"test".to_vec(),
                seq: seq.to_string().as_bytes().to_vec(),
                qual: vec![b'!'; seq.len()],
            };
            read.trim_poly_base(min_length, max_mismatch_fraction, 5, base)
                .seq
                .iter()
                .map(|&b| b as char)
                .collect()
        }
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNGNNNNNNNNNNNNNNNNNNNNNN", 25, 1./24.0, b'N'), "CTCCTGCACATCAACTTTCTNCTCATG");

        assert_eq!(&trim("AGCT", 1, 0.0, b'G'), "AGCT");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCTNNN", 1, 0.0, b'N'), "AGCT");
        assert_eq!(&trim("NGCTNNN", 1, 0.0, b'N'), "NGCT");
        assert_eq!(&trim("NNNN", 1, 0.0, b'N'), "");
        assert_eq!(&trim("AGCTNTN", 1, 1., b'N'), "AGCT");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("AGCT", 1, 0.0, b'T'), "AGC");
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN", 24, 0.0, b'N'), "CTCCTGCACATCAACTTTCTNCTCATG");
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN", 10, 0.0, b'N'), "CTCCTGCACATCAACTTTCTNCTCATG");
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN", 25, 0.0, b'N'), "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN");
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNNNNNN", 24, 0.0, b'.'), "CTCCTGCACATCAACTTTCTNCTCATG");
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT", 24, 0.0, b'.'), "CTCCTGCACATCAACTTTCTNCTCATG");
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN", 25, 0.0, b'.'), "CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN");
        //that should both be accepted at 1/24th
        assert_eq!(&trim("CTCCTGCACATCAACTTTCTNCTCATGNNNNNNNNNNNNNNNNNNNGNNNN", 25, 1./24.0, b'.'), "CTCCTGCACATCAACTTTCTNCTCATG");
    }
}

