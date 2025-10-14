status: open
# filter by expected error https://academic.oup.com/bioinformatics/article/31/21/3476/194979

So essentially, CalcExpectedError

We have some expected error calculation in reports.

Well, actually a lookup table in transformations/reports/common.rs
Q_LOOKUP (And that assumes PHRED 33 data..)
per position / phred score.

We just need ot generalize it for the whole read.

The paper says a threshold of 'Emax = 1 is a natural choise as the most probable number of errors
is zero when E <1'

E = Sum(p_i) =  Sum(10^(-Q_i/10))

That means we can just sum up our lookup table, 

