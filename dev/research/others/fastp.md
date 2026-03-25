
- Regex based barcode extractor https://crates.io/crates/barkit
  claims regex with approximate matching
  'use lowercase for fuzzy matchin'g
  - seems to use FancyRegex
  - I'm not sure what it's actually doing... seems to replace them with
    with as many patterns that have one (or more?) positions replaced with a .
    Basically pushes it down into the regex engine.
    yeah, that's exactly what it does. each lower case letter is in turn replaced with a
    '.'. something something permutation max, basically going through all 2^n possibilites,
    and if they have less ones than max-errors, replace those with '.',
    and the join all the regexs
    I mean, sure works for smallish, but 2^n is bad, and even if the number of accepted
    permutations is much lower (also this is 'pick k from n', could use something much smarther than 2^n).

-
