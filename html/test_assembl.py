from pathlib import Path
input = Path("template.html").read_text()
chart = Path("chart/chart.umd.min.js").read_text()
data = Path("output.json").read_text()
output = input.replace('"%DATA%"',  data).replace("/*%CHART%*/", chart)

p = Path('output.html')
p.write_text(output)
print("build, look at", p)


