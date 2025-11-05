use crate::config::SegmentIndex;
use anyhow::Result;
use bio::alignment::{
    AlignmentOperation,
    pairwise::{Aligner, MIN_SCORE, Scoring},
};
use bstr::BString;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct HitRegion {
    pub start: usize,
    pub len: usize,
    pub segment_index: SegmentIndex,
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
    Location(Hits),
    String(BString),
    Numeric(f64),
    Bool(bool),
}

impl TagValue {
    pub fn is_missing(&self) -> bool {
        matches!(self, TagValue::Missing)
    }

    pub fn truthy_val(&self) -> bool {
        match self {
            TagValue::Missing => false,
            TagValue::Location(_hits) => true,
            TagValue::String(_bstring) => true,
            TagValue::Numeric(_) => panic!("truthy val on numeric tags not supported"),
            TagValue::Bool(val) => *val,
        }
    }

    pub fn as_numeric(&self) -> Option<f64> {
        match self {
            TagValue::Numeric(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&Hits> {
        match self {
            TagValue::Location(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            TagValue::Bool(n) => Some(*n),
            _ => None,
        }
    }
}

impl From<f64> for TagValue {
    fn from(value: f64) -> Self {
        TagValue::Numeric(value)
    }
}

impl HitRegion {
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}
impl Hits {
    /* pub fn new(start: usize, len: usize, target: Segment) -> Self {
        Hit {
            regions: vec![HitRegion { start, len, target }],
            replacement: None,
        }
    } */

    pub fn new(start: usize, len: usize, segment_index: SegmentIndex, sequence: BString) -> Self {
        Hits(vec![Hit {
            location: Some(HitRegion {
                start,
                len,
                segment_index,
            }),
            sequence,
        }])
    }

    pub fn new_without_location(sequence: BString) -> Self {
        Hits(vec![Hit {
            location: None,
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

    pub fn covered_len(&self) -> usize {
        let mut total = 0;
        for hit in &self.0 {
            if let Some(loc) = &hit.location {
                total += loc.len;
            }
        }
        total
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
    segment: SegmentIndex,
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
                    segment,
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
                    segment,
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
                        segment,
                        reference[start..start + query.len()].into(),
                    ));
                }
                None => return None,
            }
        }
    }
    None
}

#[inline]
fn iupac_alignment_score(a: u8, b: u8) -> i32 {
    if iupac_hamming_distance(&[a], &[b]) == 0 {
        1
    } else {
        -1
    }
}

pub fn find_iupac_with_indel(
    reference: &[u8],
    query: &[u8],
    anchor: Anchor,
    max_mismatches: usize,
    max_indel_bases: usize,
    max_total_edits: Option<usize>,
    segment: SegmentIndex,
) -> Option<Hits> {
    if query.is_empty() || reference.is_empty() {
        return None;
    }

    let total_limit = max_total_edits.unwrap_or(max_mismatches + max_indel_bases);

    // Fast length checks to avoid unnecessary alignments.
    if reference.len() + max_indel_bases < query.len() {
        return None;
    }

    let query_upper: Vec<u8> = query.iter().map(u8::to_ascii_uppercase).collect();
    let reference_upper: Vec<u8> = reference.iter().map(u8::to_ascii_uppercase).collect();

    let base_scoring = Scoring::new(0, -1, iupac_alignment_score);
    let scoring = match anchor {
        Anchor::Left => base_scoring.yclip_prefix(MIN_SCORE).yclip_suffix(0),
        Anchor::Right => base_scoring.yclip_prefix(0).yclip_suffix(MIN_SCORE),
        Anchor::Anywhere => base_scoring.yclip(0),
    };

    let mut aligner = Aligner::with_scoring(scoring);
    let alignment = aligner.custom(&query_upper, &reference_upper);

    if alignment.operations.is_empty() {
        return None;
    }

    match anchor {
        Anchor::Left if alignment.ystart != 0 => return None,
        Anchor::Right if alignment.yend != reference.len() => return None,
        _ => {}
    }

    let mut mismatches = 0usize;
    let mut insertions = 0usize;
    let mut deletions = 0usize;

    for op in &alignment.operations {
        match op {
            AlignmentOperation::Subst => mismatches += 1,
            AlignmentOperation::Del => insertions += 1,
            AlignmentOperation::Ins => deletions += 1,
            AlignmentOperation::Match
            | AlignmentOperation::Xclip(_)
            | AlignmentOperation::Yclip(_) => {}
        }
    }

    let total_indels = insertions + deletions;
    if mismatches > max_mismatches
        || total_indels > max_indel_bases
        || mismatches + total_indels > total_limit
    {
        return None;
    }

    let start = alignment.ystart;
    let end = alignment.yend;

    if end <= start || end > reference.len() {
        return None;
    }

    Some(Hits::new(
        start,
        end - start,
        segment,
        reference[start..end].into(),
    ))
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

/// Reverse complement a DNA sequence
/// Handles standard bases (ATCGN) in upper and lowercase
pub fn reverse_complement(seq: &[u8]) -> Vec<u8> {
    seq.iter()
        .rev()
        .map(|&base| match base {
            b'A' => b'T',
            b'T' => b'A',
            b'C' => b'G',
            b'G' => b'C',
            b'N' => b'N',
            // Handle lowercase as well
            b'a' => b't',
            b't' => b'a',
            b'c' => b'g',
            b'g' => b'c',
            b'n' => b'n',
            _ => base, // Pass through other characters
        })
        .collect()
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

/// Check if two IUPAC barcode patterns can accept the same sequence
pub fn iupac_overlapping(pattern1: &[u8], pattern2: &[u8]) -> bool {
    // Different lengths cannot overlap
    if pattern1.len() != pattern2.len() {
        return false;
    }

    // Check each position - patterns overlap if all positions are compatible
    for (c1, c2) in pattern1.iter().zip(pattern2.iter()) {
        if !positions_compatible(*c1, *c2) {
            return false;
        }
    }
    true
}

/// Check if two IUPAC positions have overlapping base sets
fn positions_compatible(c1: u8, c2: u8) -> bool {
    let set1 = iupac_to_bases(c1);
    for base2 in iupac_to_bases(c2) {
        if set1.contains(base2) {
            return true;
        }
    }
    false
}

/// Convert an IUPAC character to its set of possible bases
fn iupac_to_bases(c: u8) -> &'static [u8] {
    match c.to_ascii_uppercase() {
        b'A' => b"A",
        b'C' => b"C",
        b'G' => b"G",
        b'T' | b'U' => b"T",
        b'R' => b"AG",
        b'Y' => b"CT",
        b'S' => b"GC",
        b'W' => b"AT",
        b'K' => b"GT",
        b'M' => b"AC",
        b'B' => b"CGT",
        b'D' => b"AGT",
        b'H' => b"ACT",
        b'V' => b"ACG",
        b'N' => b"ACGT",
        b'_' => b"", // treat _ as ignored
        _ => panic!("non iupac string passed to iupac_to_bases"),
    }
}

#[cfg(test)]
mod test {
    use crate::config::SegmentIndex;

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
            super::find_iupac(b"AGTTC", b"AGT", super::Anchor::Left, 0, SegmentIndex(0)),
            Some(super::Hits::new(0, 3, SegmentIndex(0), b"AGT".into()))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"TTC", super::Anchor::Right, 0, SegmentIndex(1)),
            Some(super::Hits::new(2, 3, SegmentIndex(1), "TTC".into()))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GT", super::Anchor::Anywhere, 0, SegmentIndex(2)),
            Some(super::Hits::new(1, 2, SegmentIndex(2), b"GT".into()))
        );
        assert_eq!(
            super::find_iupac(
                b"AGTTC",
                b"AGT",
                super::Anchor::Anywhere,
                0,
                SegmentIndex(2)
            ),
            Some(super::Hits::new(0, 3, SegmentIndex(2), b"AGT".into()))
        );
        assert_eq!(
            super::find_iupac(
                b"AGTTC",
                b"TTC",
                super::Anchor::Anywhere,
                0,
                SegmentIndex(2)
            ),
            Some(super::Hits::new(2, 3, SegmentIndex(2), b"TTC".into(),))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GT", super::Anchor::Left, 0, SegmentIndex(1)),
            None
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GT", super::Anchor::Right, 0, SegmentIndex(1)),
            None
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"GG", super::Anchor::Anywhere, 0, SegmentIndex(1)),
            None,
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"T", super::Anchor::Anywhere, 0, SegmentIndex(1)),
            Some(super::Hits::new(
                //first hit reported.
                2,
                1,
                SegmentIndex(1),
                b"T".into()
            ))
        );
        assert_eq!(
            super::find_iupac(b"AGTTC", b"AA", super::Anchor::Left, 1, SegmentIndex(1)),
            Some(super::Hits::new(0, 2, SegmentIndex(1), b"AG".into(),))
        );
    }

    #[test]
    fn test_find_iupac_with_indel() {
        // Perfect match behaves like the mismatch-only variant.
        assert_eq!(
            super::find_iupac_with_indel(
                b"AGTTC",
                b"AGT",
                super::Anchor::Anywhere,
                0,
                0,
                None,
                SegmentIndex(0),
            ),
            Some(super::Hits::new(0, 3, SegmentIndex(0), b"AGT".into()))
        );

        // Allow a single substitution.
        assert_eq!(
            super::find_iupac_with_indel(
                b"AGTTC",
                b"AAT",
                super::Anchor::Left,
                1,
                0,
                None,
                SegmentIndex(2),
            ),
            Some(super::Hits::new(0, 3, SegmentIndex(2), b"AGT".into()))
        );

        // Allow an insertion in the reference (extra base in the read).
        assert_eq!(
            super::find_iupac_with_indel(
                b"AGGTC",
                b"AGTC",
                super::Anchor::Anywhere,
                0,
                1,
                None,
                SegmentIndex(3),
            ),
            Some(super::Hits::new(0, 5, SegmentIndex(3), b"AGGTC".into()))
        );

        // Allow a deletion in the reference (missing base in the read).
        assert_eq!(
            super::find_iupac_with_indel(
                b"AGTC",
                b"AGGTC",
                super::Anchor::Anywhere,
                0,
                1,
                None,
                SegmentIndex(4),
            ),
            Some(super::Hits::new(0, 4, SegmentIndex(4), b"AGTC".into()))
        );

        // Enforce anchoring at the left edge.
        assert_eq!(
            super::find_iupac_with_indel(
                b"CCAGTTC",
                b"AGT",
                super::Anchor::Left,
                0,
                1,
                None,
                SegmentIndex(5),
            ),
            None
        );

        // Reject when mismatches exceed the dedicated limit.
        assert_eq!(
            super::find_iupac_with_indel(
                b"AGGTC",
                b"AATC",
                super::Anchor::Anywhere,
                0,
                1,
                None,
                SegmentIndex(6),
            ),
            None
        );

        // Respect the total edit budget when provided.
        assert_eq!(
            super::find_iupac_with_indel(
                b"AGGTC",
                b"AATC",
                super::Anchor::Anywhere,
                1,
                1,
                Some(1),
                SegmentIndex(7),
            ),
            None
        );
    }

    use super::*;

    #[test]
    fn test_positions_compatible() {
        // Same base should be compatible
        assert!(positions_compatible(b'A', b'A'));
        assert!(positions_compatible(b'T', b'T'));

        // Different bases should not be compatible
        assert!(!positions_compatible(b'A', b'T'));
        assert!(!positions_compatible(b'C', b'G'));

        // IUPAC codes should be compatible with their bases
        assert!(positions_compatible(b'R', b'A'));
        assert!(positions_compatible(b'R', b'G'));
        assert!(!positions_compatible(b'R', b'C'));
        assert!(!positions_compatible(b'R', b'T'));

        // N should be compatible with everything
        assert!(positions_compatible(b'N', b'A'));
        assert!(positions_compatible(b'N', b'T'));
        assert!(positions_compatible(b'N', b'C'));
        assert!(positions_compatible(b'N', b'G'));
    }

    #[test]
    fn test_iupac_overlapping() {
        // Different lengths should not overlap
        assert!(!iupac_overlapping(b"AT", b"ATC"));

        // Same sequence should overlap
        assert!(iupac_overlapping(b"ATCG", b"ATCG"));

        // Different sequences should not overlap
        assert!(!iupac_overlapping(b"ATCG", b"GGCC"));

        // IUPAC overlaps
        assert!(iupac_overlapping(b"NNNN", b"ATCG"));
        assert!(iupac_overlapping(b"ATCG", b"NNNN"));
        assert!(iupac_overlapping(b"ATVG", b"ATCG")); // A-T-[A/C/G]-G vs A-T-C-G
        assert!(iupac_overlapping(b"ATCG", b"ATCN"));
        assert!(iupac_overlapping(b"N", b"A"));
        assert!(iupac_overlapping(b"N", b"G"));
        assert!(iupac_overlapping(b"N", b"C"));
        assert!(iupac_overlapping(b"N", b"T"));
        assert!(iupac_overlapping(b"R", b"A"));

        assert!(iupac_overlapping(b"R", b"A"));
        assert!(iupac_overlapping(b"R", b"G"));
        assert!(!iupac_overlapping(b"R", b"C"));
        assert!(!iupac_overlapping(b"R", b"T"));

        assert!(iupac_overlapping(b"Y", b"C"));
        assert!(iupac_overlapping(b"Y", b"T"));
        assert!(!iupac_overlapping(b"Y", b"A"));
        assert!(!iupac_overlapping(b"Y", b"G"));

        assert!(iupac_overlapping(b"S", b"G"));
        assert!(iupac_overlapping(b"S", b"C"));
        assert!(!iupac_overlapping(b"S", b"A"));
        assert!(!iupac_overlapping(b"S", b"T"));

        assert!(iupac_overlapping(b"W", b"A"));
        assert!(iupac_overlapping(b"W", b"T"));
        assert!(!iupac_overlapping(b"W", b"G"));
        assert!(!iupac_overlapping(b"W", b"C"));

        assert!(iupac_overlapping(b"K", b"G"));
        assert!(iupac_overlapping(b"K", b"T"));
        assert!(!iupac_overlapping(b"K", b"A"));
        assert!(!iupac_overlapping(b"K", b"C"));

        assert!(iupac_overlapping(b"M", b"A"));
        assert!(iupac_overlapping(b"M", b"C"));
        assert!(!iupac_overlapping(b"M", b"G"));
        assert!(!iupac_overlapping(b"M", b"T"));

        assert!(iupac_overlapping(b"B", b"C"));
        assert!(iupac_overlapping(b"B", b"G"));
        assert!(iupac_overlapping(b"B", b"T"));
        assert!(!iupac_overlapping(b"B", b"A"));

        assert!(iupac_overlapping(b"D", b"A"));
        assert!(iupac_overlapping(b"D", b"G"));
        assert!(iupac_overlapping(b"D", b"T"));
        assert!(!iupac_overlapping(b"D", b"C"));

        assert!(iupac_overlapping(b"H", b"A"));
        assert!(iupac_overlapping(b"H", b"C"));
        assert!(iupac_overlapping(b"H", b"T"));
        assert!(!iupac_overlapping(b"H", b"G"));

        assert!(iupac_overlapping(b"V", b"A"));
        assert!(iupac_overlapping(b"V", b"C"));
        assert!(iupac_overlapping(b"V", b"G"));
        assert!(!iupac_overlapping(b"V", b"T"));

        assert!(iupac_overlapping(b"U", b"T"));
        assert!(iupac_overlapping(b"U", b"U"));
        assert!(!iupac_overlapping(b"U", b"C"));
        assert!(!iupac_overlapping(b"U", b"G"));
        assert!(!iupac_overlapping(b"U", b"A"));

        // Non-overlapping IUPAC
        assert!(!iupac_overlapping(b"RYRY", b"ATCG")); // R=A/G, Y=C/T vs A-T-C-G
    }
}
