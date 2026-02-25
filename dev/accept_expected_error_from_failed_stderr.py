import os
from pathlib import Path


def process_stderr_files(base_dir="test_cases"):
    base_path = Path(base_dir)

    # Find all 'stderr' files under 'test_cases'
    for stderr_file in base_path.rglob("stderr"):
        if stderr_file.is_file():
            try:
                # Read stderr content
                stderr_content = stderr_file.read_text(encoding="utf-8")

                # Find first occurrence of 'Error'
                error_idx = stderr_content.find("Error")
                if error_idx == -1:
                    print(f"No 'Error' found in {stderr_file}")
                    continue

                # Extract from first 'Error' to end
                error_text = stderr_content[error_idx:]

                # Go up one folder from stderr's directory
                parent_dir = stderr_file.parent.parent
                expected_error_file = parent_dir / "expected_error.txt"

                # If expected_error.txt exists, update its contents
                if expected_error_file.exists():
                    expected_error_file.write_text(error_text, encoding="utf-8")
                    print(f"Updated: {expected_error_file}")
                else:
                    print(f"Skipping (no expected_error.txt in {parent_dir})")

            except Exception as e:
                print(f"Error processing {stderr_file}: {e}")


# Run it
if __name__ == "__main__":
    process_stderr_files()
