# This script copies changed 'output.json' and 'output.html' files
# when we've mass changed something again...
from pathlib import Path
import shutil



for query in ['output.json','output.html']:
    for fn in Path('test_cases').rglob(query):
        if fn.parent.name == 'actual':
            out_fn = fn.parent.parent / query
            a = fn.read_text()
            try:
                b = out_fn.read_text()
            except OSError:
                b = ''
            if a != b:
                print("copying", fn, "to", out_fn)
                shutil.copyfile(fn, out_fn)

print("Done.")
