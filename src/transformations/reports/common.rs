use super::super::InputInfo;

pub const PHRED33OFFSET: u8 = 33;

// phred score (33 sanger encoding) to probability of error
// python: ([1.0] * 32 + [10**(q/-10) for q in range(0,256)])[:256]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::excessive_precision)]
pub const Q_LOOKUP: [f64; 256] = [
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    1.0,
    0.7943282347242815,
    0.6309573444801932,
    0.5011872336272722,
    0.3981071705534972,
    0.31622776601683794,
    0.251188643150958,
    0.19952623149688797,
    0.15848931924611134,
    0.12589254117941673,
    0.1,
    0.07943282347242814,
    0.06309573444801933,
    0.05011872336272722,
    0.039810717055349734,
    0.03162277660168379,
    0.025118864315095794,
    0.0199526231496888,
    0.015848931924611134,
    0.012589254117941675,
    0.01,
    0.007943282347242814,
    0.00630957344480193,
    0.005011872336272725,
    0.003981071705534973,
    0.0031622776601683794,
    0.0025118864315095794,
    0.001995262314968879,
    0.001584893192461114,
    0.0012589254117941675,
    0.001,
    0.0007943282347242813,
    0.000630957344480193,
    0.0005011872336272725,
    0.00039810717055349735,
    0.00031622776601683794,
    0.00025118864315095795,
    0.00019952623149688788,
    0.00015848931924611142,
    0.00012589254117941674,
    0.0001,
    7.943282347242822e-05,
    6.309573444801929e-05,
    5.011872336272725e-05,
    3.9810717055349695e-05,
    3.1622776601683795e-05,
    2.5118864315095822e-05,
    1.9952623149688786e-05,
    1.584893192461114e-05,
    1.2589254117941661e-05,
    1e-05,
    7.943282347242822e-06,
    6.30957344480193e-06,
    5.011872336272725e-06,
    3.981071705534969e-06,
    3.162277660168379e-06,
    2.5118864315095823e-06,
    1.9952623149688787e-06,
    1.584893192461114e-06,
    1.2589254117941661e-06,
    1e-06,
    7.943282347242822e-07,
    6.30957344480193e-07,
    5.011872336272725e-07,
    3.981071705534969e-07,
    3.162277660168379e-07,
    2.5118864315095823e-07,
    1.9952623149688787e-07,
    1.584893192461114e-07,
    1.2589254117941662e-07,
    1e-07,
    7.943282347242822e-08,
    6.30957344480193e-08,
    5.011872336272725e-08,
    3.981071705534969e-08,
    3.162277660168379e-08,
    2.511886431509582e-08,
    1.9952623149688786e-08,
    1.5848931924611143e-08,
    1.2589254117941661e-08,
    1e-08,
    7.943282347242822e-09,
    6.309573444801943e-09,
    5.011872336272715e-09,
    3.981071705534969e-09,
    3.1622776601683795e-09,
    2.511886431509582e-09,
    1.9952623149688828e-09,
    1.584893192461111e-09,
    1.2589254117941663e-09,
    1e-09,
    7.943282347242822e-10,
    6.309573444801942e-10,
    5.011872336272714e-10,
    3.9810717055349694e-10,
    3.1622776601683795e-10,
    2.511886431509582e-10,
    1.9952623149688828e-10,
    1.584893192461111e-10,
    1.2589254117941662e-10,
    1e-10,
    7.943282347242822e-11,
    6.309573444801942e-11,
    5.011872336272715e-11,
    3.9810717055349695e-11,
    3.1622776601683794e-11,
    2.5118864315095823e-11,
    1.9952623149688828e-11,
    1.5848931924611107e-11,
    1.2589254117941662e-11,
    1e-11,
    7.943282347242821e-12,
    6.309573444801943e-12,
    5.011872336272715e-12,
    3.9810717055349695e-12,
    3.1622776601683794e-12,
    2.5118864315095823e-12,
    1.9952623149688827e-12,
    1.584893192461111e-12,
    1.258925411794166e-12,
    1e-12,
    7.943282347242822e-13,
    6.309573444801942e-13,
    5.011872336272715e-13,
    3.981071705534969e-13,
    3.162277660168379e-13,
    2.511886431509582e-13,
    1.9952623149688827e-13,
    1.584893192461111e-13,
    1.2589254117941663e-13,
    1e-13,
    7.943282347242822e-14,
    6.309573444801943e-14,
    5.0118723362727144e-14,
    3.9810717055349693e-14,
    3.1622776601683796e-14,
    2.5118864315095823e-14,
    1.9952623149688828e-14,
    1.584893192461111e-14,
    1.2589254117941662e-14,
    1e-14,
    7.943282347242822e-15,
    6.309573444801943e-15,
    5.0118723362727146e-15,
    3.9810717055349695e-15,
    3.1622776601683794e-15,
    2.511886431509582e-15,
    1.995262314968883e-15,
    1.584893192461111e-15,
    1.2589254117941663e-15,
    1e-15,
    7.943282347242821e-16,
    6.309573444801943e-16,
    5.011872336272715e-16,
    3.9810717055349695e-16,
    3.1622776601683793e-16,
    2.511886431509582e-16,
    1.995262314968883e-16,
    1.5848931924611109e-16,
    1.2589254117941662e-16,
    1e-16,
    7.943282347242789e-17,
    6.309573444801943e-17,
    5.0118723362727144e-17,
    3.9810717055349855e-17,
    3.1622776601683796e-17,
    2.5118864315095718e-17,
    1.9952623149688827e-17,
    1.584893192461111e-17,
    1.2589254117941713e-17,
    1e-17,
    7.94328234724279e-18,
    6.309573444801943e-18,
    5.011872336272715e-18,
    3.981071705534985e-18,
    3.1622776601683795e-18,
    2.5118864315095718e-18,
    1.995262314968883e-18,
    1.5848931924611109e-18,
    1.2589254117941713e-18,
    1e-18,
    7.943282347242789e-19,
    6.309573444801943e-19,
    5.011872336272715e-19,
    3.9810717055349853e-19,
    3.162277660168379e-19,
    2.5118864315095717e-19,
    1.995262314968883e-19,
    1.584893192461111e-19,
    1.2589254117941713e-19,
    1e-19,
    7.94328234724279e-20,
    6.309573444801943e-20,
    5.011872336272715e-20,
    3.9810717055349855e-20,
    3.162277660168379e-20,
    2.511886431509572e-20,
    1.9952623149688828e-20,
    1.5848931924611108e-20,
    1.2589254117941713e-20,
    1e-20,
    7.943282347242789e-21,
    6.309573444801943e-21,
    5.011872336272714e-21,
    3.981071705534986e-21,
    3.1622776601683792e-21,
    2.511886431509572e-21,
    1.9952623149688827e-21,
    1.5848931924611108e-21,
    1.2589254117941713e-21,
    1e-21,
    7.943282347242789e-22,
    6.309573444801943e-22,
    5.011872336272715e-22,
    3.9810717055349856e-22,
    3.1622776601683793e-22,
    2.511886431509572e-22,
    1.9952623149688828e-22,
    1.584893192461111e-22,
    1.2589254117941713e-22,
    1e-22,
    7.943282347242789e-23,
    6.309573444801943e-23,
    5.011872336272715e-23,
];

pub const BASE_TO_INDEX: [u8; 256] = {
    let mut out = [4; 256]; //everything else is an N
    out[b'A' as usize] = 0;
    out[b'C' as usize] = 1;
    out[b'G' as usize] = 2;
    out[b'T' as usize] = 3;
    out[b'a' as usize] = 0;
    out[b'c' as usize] = 1;
    out[b'g' as usize] = 2;
    out[b't' as usize] = 3;
    out
};

pub fn default_progress_n() -> usize {
    1_000_000
}

pub fn default_true() -> bool {
    true
}

///turn a float into a string with thousands formatting
///and arbirtrary post-decimal digits
pub fn thousands_format(value: f64, digits: u8) -> String {
    let str = format!("{value:.*}", digits as usize);
    let parts: Vec<&str> = str.split('.').collect();
    let mut integer_part = Vec::new();
    for (ii, char) in parts[0].chars().rev().enumerate() {
        if ii > 0 && ii % 3 == 0 {
            integer_part.push('_');
        }
        integer_part.push(char);
    }
    integer_part.reverse();
    let integer_part: String = integer_part.into_iter().collect();
    if digits > 0 {
        format!("{}.{}", integer_part, parts.get(1).unwrap_or(&""))
    } else {
        integer_part
    }
}

#[derive(Clone, Debug)]
pub struct PositionCount(pub [usize; 5]);

#[derive(serde::Serialize, Debug, Clone, Default)]
pub struct PositionCountOut {
    pub a: Vec<usize>,
    pub c: Vec<usize>,
    pub g: Vec<usize>,
    pub t: Vec<usize>,
    pub n: Vec<usize>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ReportData<T> {
    pub program_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read1: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read2: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index1: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index2: Option<T>,
}

impl<T> Default for ReportData<T> {
    fn default() -> Self {
        ReportData {
            program_version: env!("CARGO_PKG_VERSION").to_string(),
            read1: None,
            read2: None,
            index1: None,
            index2: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PerReadReportData<T> {
    pub read1: Option<T>,
    pub read2: Option<T>,
    pub index1: Option<T>,
    pub index2: Option<T>,
}

impl<T: std::default::Default> PerReadReportData<T> {
    pub fn new(input_info: &InputInfo) -> Self {
        Self {
            read1: if input_info.has_read1 {
                Some(Default::default())
            } else {
                None
            },
            read2: if input_info.has_read2 {
                Some(Default::default())
            } else {
                None
            },

            index1: if input_info.has_index1 {
                Some(Default::default())
            } else {
                None
            },

            index2: if input_info.has_index2 {
                Some(Default::default())
            } else {
                None
            },
        }
    }
}

impl<T: Into<serde_json::Value> + Clone> PerReadReportData<T> {
    pub fn store(&self, key: &str, target: &mut serde_json::Map<String, serde_json::Value>) {
        if let Some(read1) = &self.read1 {
            let entry = target
                .entry("read1".to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
            entry
                .as_object_mut()
                .unwrap()
                .insert(key.to_string(), (read1.to_owned()).into());
        }
        if let Some(read2) = &self.read2 {
            let entry = target
                .entry("read2".to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
            entry
                .as_object_mut()
                .unwrap()
                .insert(key.to_string(), (read2.to_owned()).into());
        }
        if let Some(index1) = &self.index1 {
            let entry = target
                .entry("index1".to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
            entry
                .as_object_mut()
                .unwrap()
                .insert(key.to_string(), (index1.to_owned()).into());
        }
        if let Some(index2) = &self.index2 {
            let entry = target
                .entry("index2".to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
            entry
                .as_object_mut()
                .unwrap()
                .insert(key.to_string(), (index2.to_owned()).into());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::thousands_format;

    #[test]
    fn test_thousands_format() {
        assert_eq!(thousands_format(0., 0), "0");
        assert_eq!(thousands_format(1000., 0), "1_000");
        assert_eq!(thousands_format(10000., 0), "10_000");
        assert_eq!(thousands_format(10000.12, 2), "10_000.12");
        assert_eq!(thousands_format(100_000_000.12, 2), "100_000_000.12");
        assert_eq!(thousands_format(5.12, 2), "5.12");
    }
}
