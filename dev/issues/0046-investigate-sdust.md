status: open
# investigate SDUST

 [SDUST algorithm](https://pubmed.ncbi.nlm.nih.gov/16796549/), perhaps by linking in @lh3's [standalone C implementation](https://github.com/lh3/sdust)?
(https://github.com/OpenGene/fastp/issues/216)

(also) A low complexity filter using methods such as Entropy or Dust. The current filter does not work well on tandem repeats and similar type of low complexity sequences.
(https://github.com/OpenGene/fastp/issues/332)

The symetric 'dust' algorithm to identify low-complexity regions.

Paper behind paywall.

Here's the chatGTP description of the algorithm, beware:

The Symmetric DUST algorithm is a heuristic method for masking low-complexity regions in DNA sequences—segments dominated by short, repetitive patterns that can confound alignment and database searches.

Core idea:
It quantifies local sequence complexity by evaluating the frequency of short nucleotide triplets (3-mers) within a sliding window, assigning a “complexity score” that increases with repetitiveness.

Algorithm steps

Triplet counting:
For each window (typically 64 bp), count occurrences of all 3-mers.

Score computation:
For each unique triplet with count 
n
n, add:

s=n(n−1)/2
s=n(n−1)/2

to the window’s score.
This measures redundancy — perfectly random sequences yield low scores; repetitive ones high.

Normalization:
Normalize by window size and compare against a threshold (e.g., 20).
Regions exceeding the threshold are considered low-complexity.

Symmetric masking:
When a region is masked, both forward and backward directions are evaluated—hence “symmetric”—to avoid bias at window edges and ensure consistent masking regardless of strand orientation.

Output:
Bases within low-complexity segments are replaced (e.g., with N or lowercase letters), preserving sequence length.

Reference:
Tatusov & Lipman, DUST: Masking low-complexity DNA sequences, NCBI technical report (1994); implemented in BLAST and dustmasker.

