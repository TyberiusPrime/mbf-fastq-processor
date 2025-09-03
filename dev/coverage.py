#!/usr/bin/env python3
"""
Coverage collection script for mbf_fastq_processor

This script runs cargo llvm-cov and generates coverage reports in multiple formats.
"""

import argparse
import subprocess
import sys
from pathlib import Path


def run_command(cmd, description):
    """Run a command and handle errors"""
    print(f"🔄 {description}...")
    try:
        result = subprocess.run(
            cmd, check=True, shell=True, capture_output=True, text=True
        )
        print(f"✅ {description} completed")
        return result
    except subprocess.CalledProcessError as e:
        print(f"❌ {description} failed with exit code {e.returncode}")
        print(f"stdout: {e.stdout}")
        print(f"stderr: {e.stderr}")
        sys.exit(1)


def main():
    parser = argparse.ArgumentParser(
        description="Generate coverage reports for mbf_fastq_processor"
    )
    parser.add_argument(
        "--html", action="store_true", help="Generate HTML coverage report"
    )
    parser.add_argument(
        "--lcov", action="store_true", help="Generate LCOV coverage report"
    )
    parser.add_argument(
        "--json", action="store_true", help="Generate JSON coverage report"
    )
    parser.add_argument(
        "--summary", action="store_true", help="Show coverage summary only"
    )
    parser.add_argument(
        "--all", action="store_true", help="Generate all report formats"
    )
    parser.add_argument(
        "--open",
        action="store_true",
        help="Open HTML report in browser after generation",
    )

    args = parser.parse_args()

    # Default to summary if no specific format requested
    if not any([args.html, args.lcov, args.json, args.all, args.summary]):
        args.summary = True

    # Check if cargo-llvm-cov is installed
    try:
        subprocess.run(
            ["cargo", "llvm-cov", "--version"], check=True, capture_output=True
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("❌ cargo-llvm-cov is not installed. Please install it with:")
        print("   cargo install cargo-llvm-cov")
        sys.exit(1)

    # Change to project root
    project_root = Path(__file__).parent.parent
    print(f"📂 Running coverage from: {project_root}")

    if args.summary or args.all:
        run_command("cargo llvm-cov test --summary-only", "Generating coverage summary")

    if args.html or args.all:
        run_command(
            "cargo llvm-cov test --html --output-dir coverage-html",
            "Generating HTML coverage report",
        )
        print("📊 HTML report saved to: coverage-html/html/index.html")

        if args.open:
            try:
                import webbrowser

                html_path = project_root / "coverage-html" / "html" / "index.html"
                webbrowser.open(f"file://{html_path.absolute()}")
                print("🌐 Opening HTML report in browser...")
            except Exception as e:
                print(f"⚠️  Could not open browser: {e}")

    if args.lcov or args.all:
        run_command(
            "cargo llvm-cov test --lcov --output-path coverage.lcov",
            "Generating LCOV coverage report",
        )
        print("📊 LCOV report saved to: coverage.lcov")

    if args.json or args.all:
        run_command(
            "cargo llvm-cov test --json --output-path coverage.json",
            "Generating JSON coverage report",
        )
        print("📊 JSON report saved to: coverage.json")

    print("✅ Coverage collection completed successfully!")


if __name__ == "__main__":
    main()
