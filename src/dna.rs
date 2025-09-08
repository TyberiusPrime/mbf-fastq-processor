use crate::config::Target;
use bstr::BString;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HitRegion {
    pub start: usize,
    pub len: usize,
    pub target: Target,
}

/// a hit within a sequence.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Hit {
    pub location: Option<HitRegion>,
    pub sequence: BString,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Hits(pub Vec<Hit>);

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TagValue {
    #[default]
    Missing,
    Sequence(Hits),
    Numeric(f64),
}

impl TagValue {
    pub fn is_missing(&self) -> bool {
        matches!(self, TagValue::Missing)
    }

    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            TagValue::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&Hits> {
        match self {
            TagValue::Sequence(h) => Some(h),
            _ => None,
        }
    }
}

impl HitRegion {
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
impl Hits {
    /* pub fn new(start: usize, len: usize, target: Target) -> Self {
        Hit {
            regions: vec![HitRegion { start, len, target }],
            replacement: None,
        }
    } */

    pub fn new(start: usize, len: usize, target: Target, sequence: BString) -> Self {
        Hits(vec![Hit {
            location: Some(HitRegion { start, len, target }),
            sequence,
        }])
    }

    pub fn new_multiple(regions: Vec<Hit>) -> Self {
        Hits(regions)
    }

    pub fn joined_sequence(&self, separator: Option<&[u8]>) -> Vec<u8> {
        let mut res = Vec::new();
        let mut first = true;
        for region in &self.0 {
            if !first {
                if let Some(separator) = separator {
                    res.extend(separator.iter().copied());
                }
            }
            first = false;
            res.extend_from_slice(&region.sequence);
        }
        res
    }

    /* pub fn replacement_or_seq<'a>(&'a self, seq: &'a [u8]) -> &'a [u8] {
        if let Some(sequence) = self.sequence.as_ref() {
            sequence
        } else {
            assert!(self.regions.len() == 0, "Hit has no sequence, but multiple regions. That needs to be prevented when creating the read. Use new_with_replacemetn");
            &seq[self.regions[0].start..self.regions[0].start + self.regions[0].len]
        }
    } */
}

/// Where to search
#[derive(eserde::Deserialize, Debug, Copy, Clone)]
pub enum Anchor {
    #[serde(alias = "left")]
    Left,
    #[serde(alias = "right")]
    Right,
    #[serde(alias = "anywhere")]
    Anywhere,
}

pub fn find_iupac(
    reference: &[u8],
    query: &[u8],
    anchor: Anchor,
    max_mismatches: u8,
    target: Target,
) -> Option<Hits> {
    if reference.len() < query.len() {
        return None;
    }
    match anchor {
        Anchor::Left => {
            let hd = iupac_hamming_distance(query, reference[..query.len()].as_ref());
            if hd <= max_mismatches as usize {
                return Some(Hits::new(
                    0,
                    query.len(),
                    target,
                    reference[..query.len()].into(),
                ));
            }
        }
        Anchor::Right => {
            let hd =
                iupac_hamming_distance(query, reference[reference.len() - query.len()..].as_ref());
            if hd <= max_mismatches as usize {
                return Some(Hits::new(
                    reference.len() - query.len(),
                    query.len(),
                    target,
                    reference[reference.len() - query.len()..].into(),
                ));
            }
        }
        Anchor::Anywhere => {
            //TODO: document that we always find the first one!
            //todo: This probably could use a much faster algorithm.
            match iupac_find_best(query, reference, max_mismatches as usize) {
                Some(start) => {
                    return Some(Hits::new(
                        start,
                        query.len(),
                        target,
                        reference[start..start + query.len()].into(),
                    ));
                }
                None => return None,
            }
        }
    }
    None
}

///find the best hit for this iupac string, on parity, earlier hits prefered
pub fn iupac_find_best(query: &[u8], reference: &[u8], max_mismatches: usize) -> Option<usize> {
    let query_len = query.len();
    let mut best_pos = None;
    let mut best_so_far = max_mismatches + 1;
    for start in 0..=reference.len() - query_len {
        let hd = iupac_hamming_distance(query, &reference[start..start + query_len]);
        if hd == 0 {
            return Some(start);
        } else if hd < best_so_far {
            best_so_far = hd;
            best_pos = Some(start);
        }
    }
    best_pos
}

///
/// check if any of the extend iupac characters occurs.
pub fn contains_iupac_ambigous(input: &[u8]) -> bool {
    input.iter().any(|&char| {
        matches!(
            char,
            b'R' | b'Y' | b'S' | b'W' | b'K' | b'M' | b'B' | b'V' | b'D' | b'H' | b'N'
        )
    })
}
pub fn all_iupac(input: &[u8]) -> bool {
    input.iter().all(|&char| {
        matches!(
            char,
            b'A' | b'T'
                | b'U'
                | b'C'
                | b'G'
                | b'R'
                | b'Y'
                | b'S'
                | b'W'
                | b'K'
                | b'M'
                | b'B'
                | b'V'
                | b'D'
                | b'H'
                | b'N'
        )
    })
}

pub fn reverse_complement_iupac(input: &[u8]) -> Vec<u8> {
    let mut new_seq = Vec::new();
    for char in input.iter().rev() {
        new_seq.push(match char {
            b'A' => b'T',
            b'T' | b'U' => b'A',
            b'C' => b'G',
            b'G' => b'C',

            b'a' => b't',
            b't' | b'u' => b'a',
            b'c' => b'g',
            b'g' => b'c',

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
            b'\n' => panic!("New line in DNA sequence"), // since that's not valid fastq!
            _ => *char,
        });
    }
    new_seq
}

pub fn iupac_hamming_distance(iupac_reference: &[u8], atcg_query: &[u8]) -> usize {
    assert_eq!(
        iupac_reference.len(),
        atcg_query.len(),
        "Reference and query must have same length"
    );
    let mut dist = 0;
    for (a, b) in iupac_reference.iter().zip(atcg_query.iter()) {
        if a != b {
            match (a, b) {
                (b'A', b'a')
                | (b'a', b'A')
                | (b'C', b'c')
                | (b'c', b'C')
                | (b'G', b'g')
                | (b'g', b'G')
                | (b'T', b't')
                | (b't', b'T')
                | (b'R' | b'r', b'A' | b'G' | b'a' | b'g')
                | (b'Y' | b'y', b'C' | b'T' | b'c' | b't')
                | (b'S' | b's', b'G' | b'C' | b'g' | b'c')
                | (b'W' | b'w', b'A' | b'T' | b'a' | b't')
                | (b'K' | b'k', b'G' | b'T' | b'g' | b't')
                | (b'M' | b'm', b'A' | b'C' | b'a' | b'c')
                | (b'B' | b'b', b'C' | b'G' | b'T' | b'c' | b'g' | b't')
                | (b'D' | b'd', b'A' | b'G' | b'T' | b'a' | b'g' | b't')
                | (b'H' | b'h', b'A' | b'C' | b'T' | b'a' | b'c' | b't')
                | (b'V' | b'v', b'A' | b'C' | b'G' | b'a' | b'c' | b'g')
                | (b'N' | b'n', _) => {}
                (_, _) => dist += 1,
            }
        }
    }
    dist
}

#[cfg(test)]
mod test {
    use crate::config::Target;

    fn check(should: &[u8], input: &[u8]) {
        let s: Vec<u8> = should.to_vec();
        assert_eq!(
            std::str::from_utf8(&s).unwrap(),
            std::str::from_utf8(&super::reverse_complement_iupac(input)).unwrap()
        );
    }

    #[test]
    fn test_rev_complement() {
        check(b"AGCT", b"AGCT");
        check(b"DHBVNKMWSRYAAGCT", b"AGCTURYSWKMNBVDH");
        check(b"dhbvnkmwsryaagct", b"agcturyswkmnbvdh");
    }
    #[test]
    #[should_panic(expected = "New line in DNA sequence")]
    fn test_rev_complement_panics_on_newline() {
        super::reverse_complement_iupac(b"AGCT\n");
    }

    #[test]
    fn test_iupac_hamming_distance() {
        assert_eq!(super::iupac_hamming_distance(b"AGCT", b"AGCT"), 0);
        assert_eq!(super::iupac_hamming_distance(b"AGCT", b"AGCA"), 1);
        assert_eq!(super::iupac_hamming_distance(b"AGCT", b"AGCG"), 1);
        assert_eq!(super::iupac_hamming_distance(b"NGCC", b"AGCC"), 0);
        assert_eq!(super::iupac_hamming_distance(b"NGCC", b"AGCT"), 1);
        assert_eq!(super::iupac_hamming_distance(b"NGCC", b"cGCT"), 1);

        assert_eq!(super::iupac_hamming_distance(b"AGKC", b"agKc"), 0); //we don't enforce no iupac
        //in query
        assert_eq!(super::iupac_hamming_distance(b"AGKC", b"agkc"), 1); //we don't enforce, but we
        //don't handle different upper/lowercase either.
        let should = vec![
            (b'R', (0, 1, 0, 1)),
            (b'Y', (1, 0, 1, 0)),
            (b'S', (1, 0, 0, 1)),
            (b'W', (0, 1, 1, 0)),
            (b'K', (1, 1, 0, 0)),
            (b'M', (0, 0, 1, 1)),
            (b'B', (1, 0, 0, 0)),
            (b'D', (0, 1, 0, 0)),
            (b'H', (0, 0, 1, 0)),
            (b'V', (0, 0, 0, 1)),
            (b'N', (0, 0, 0, 0)),
        ];
        for (letter, actg) in &should {
            let str_letter = std::str::from_utf8(&[*letter]).unwrap().to_string();
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"A"),
                actg.0,
                "wrong result {str_letter} vs A"
            );
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"C"),
                actg.1,
                "wrong result {str_letter} vs C"
            );
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"G"),
                actg.2,
                "wrong result {str_letter} vs G"
            );
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"T"),
                actg.3,
                "wrong result {str_letter} vs T"
            );

            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"a"),
                actg.0,
                "wrong result {str_letter} vs a"
            );
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"c"),
                actg.1,
                "wrong result {str_letter} vs c"
            );
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"g"),
                actg.2,
                "wrong result {str_letter} vs g"
            );
            assert_eq!(
                super::iupac_hamming_distance(&[*letter], b"t"),
                actg.3,
                "wrong result {str_letter} vs t"
            );
        }
    }

    #[test]
    fn test_find_iupac() {
        assert_eq!(
            super::find_iupac(b"AGTTC", b"AGT", super::Anchor::Left, 0, Target::Read1),
            Some(super::Hits::new(0, 3, Target::Read1, b"AGT".into(),))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"TTC", super::Anchor::Right, 0, Target::Read2),
            Some(super::Hits::new(2, 3, Target::Read2, b"TTC".into()))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GT", super::Anchor::Anywhere, 0, Target::Index1),
            Some(super::Hits::new(1, 2, Target::Index1, b"GT".into()))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"AGT", super::Anchor::Anywhere, 0, Target::Index1),
            Some(super::Hits::new(0, 3, Target::Index1, b"AGT".into()))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"TTC", super::Anchor::Anywhere, 0, Target::Index1),
            Some(super::Hits::new(2, 3, Target::Index1, b"TTC".into(),))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GT", super::Anchor::Left, 0, Target::Index1),
            None
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GT", super::Anchor::Right, 0, Target::Index1),
            None
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GG", super::Anchor::Anywhere, 0, Target::Index1),
            None,
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"T", super::Anchor::Anywhere, 0, Target::Index1),
            Some(super::Hits::new(
                //first hit reported.
                2,
                1,
                Target::Index1,
                b"T".into()
            ))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"AA", super::Anchor::Left, 1, Target::Index1),
            Some(super::Hits::new(0, 2, Target::Index1, b"AG".into(),))
        );
    }
}
