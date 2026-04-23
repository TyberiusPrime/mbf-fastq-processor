import shutil
import subprocess
from pathlib import Path


def bam_content_hash(bam_path: Path) -> str:
    result = subprocess.run(
        f"samtools view -h {bam_path.name} | md5sum",
        shell=True,
        cwd=bam_path.parent,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


changed = []
content_mismatch = []

for actual_bam in Path("test_cases").rglob("output*.bam"):
    if actual_bam.parent.name != "actual":
        continue
    expected_bam = actual_bam.parent.parent / actual_bam.name
    if not expected_bam.exists():
        continue
    if actual_bam.read_bytes() == expected_bam.read_bytes():
        continue

    actual_hash = bam_content_hash(actual_bam)
    expected_hash = bam_content_hash(expected_bam)

    if actual_hash == expected_hash:
        print(f"accepting {actual_bam} -> {expected_bam}")
        shutil.copyfile(actual_bam, expected_bam)
        changed.append(expected_bam)
        for suffix in [".bai", ".compressed.sha256"]:
            actual_side = actual_bam.parent / (actual_bam.name + suffix)
            expected_side = expected_bam.parent / (expected_bam.name + suffix)
            if actual_side.exists():
                print(f"  also accepting {actual_side.name}")
                shutil.copyfile(actual_side, expected_side)
    else:
        print(f"CONTENT DIFFERS: {expected_bam}")
        print(f"  expected: {expected_hash}")
        print(f"  actual:   {actual_hash}")
        content_mismatch.append(expected_bam)

print(f"\nAccepted {len(changed)} file(s). Content mismatches: {len(content_mismatch)}.")
