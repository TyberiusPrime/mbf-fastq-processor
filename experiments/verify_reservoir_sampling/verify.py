""" Run reservoir sampling a number of times
observe how often each element is sampled,
then compare it to a binomial distribution
"""
import tempfile
from scipy import stats
import collections
import subprocess
from pathlib import Path

reps = 1000

fn = Path("../../test_cases/sample_data/filter/input_read1_1.fq")
total = len(fn.read_text().strip().split("\n")) / 4
reads = (fn.read_text().strip().split("\n"))[::4]
choose_rate = 0.2
chosen = int(total * choose_rate)


input_toml = f"""
[input]
    read1 = "{str(fn)}"
[output]
    stdout = true
    prefix = 'ignored'
[[step]]
    action = "FilterReservoirSample"
    n = {chosen}
    """

seen = collections.Counter()

subprocess.check_call(["cargo", "build", "--release"], cwd="../../")

for ii in range(reps):
    if ii % 100 == 0:
        print(".", end="", flush=True)
    tf = tempfile.NamedTemporaryFile(mode="w+", suffix=".toml", delete=True)
    tf.write(input_toml + f"seed = {ii * 2 + 100}")
    tf.flush()
    out = subprocess.check_output(
        ["../../target/release/mbf-fastq-processor", "process", tf.name]
    ).decode()
    lines = out.strip().split("\n")
    read_names = lines[::4]
    for name in read_names:
        seen[name] += 1
    assert len(read_names) == chosen


excepted = reps * chosen / total
# guess I assume a normal distribution here
sorted_seen = sorted(seen.items(), key=lambda k: k[1])
print('range', sorted_seen[0], sorted_seen[-1][0])

print('total unique observed element count', len(seen))
print('total count', sum(seen.values()))
mean_rate = sum(seen.values()) / (total * reps)
print("overall mean rate:", mean_rate)

theoretical_dist = stats.binom(n=reps, p=choose_rate)
ks_statistic, p_value = stats.kstest(list(seen.values()), 
                                     lambda x: theoretical_dist.cdf(x))
print(f"KS test p-value: {p_value:.4f}")
print("(p > 0.05 means data is consistent with binomial)")
print("And breaking the R algorithm by adjusting the range = 1..i instead of 1..=i does get it to 0.03")




print("should be tight like third digit after decimal")
