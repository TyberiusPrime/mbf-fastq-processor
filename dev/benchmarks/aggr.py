import sys
from pathlib import Path

fn = Path(sys.argv[1])
means = {}
for line in fn.read_text().splitlines():
    line = line.strip()
    if line.startswith("Benchmarking"):
        parts = line.split()
        name = parts[1]
    elif line.startswith("step_benchmarks/pipeline/"):
        name = line[len("step_benchmarks/pipeline/") :].strip()
    elif line.startswith("time:   "):
        times = line[line.find("[") + 1 : line.rfind("]")].split(" ")
        times_and_units = [(times[x], times[x + 1]) for x in range(0, len(times), 2)]
        assert len(times_and_units) == 3
        middle = times_and_units[1]
        if middle[1] == "ms":
            means[name] = float(middle[0])
        elif middle[1] == "s":
            means[name] = float(middle[0]) * 1000
        else:
            raise ValueError("unknown time unit")

means = list(means.items())
means.sort(key=lambda x: x[1])
for name, mean in means[::-1]:
    print(f"{mean:2.2f}ms {name}")
