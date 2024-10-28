from pathlib import Path
input = Path("template.html").read_text()
chart = Path("chart/chart.umd.min.js").read_text()
data = Path("output_report.json").read_text()
output = input.replace('"%DATA%"',  data).replace("/*%CHART%*/", chart)

Path('output.html').write_text(output)
print("build")


