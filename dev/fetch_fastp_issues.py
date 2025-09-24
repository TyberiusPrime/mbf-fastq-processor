#!/usr/bin/env python3
"""
Fetch all GitHub issues from OpenGene/fastp repository.
"""

import json
import subprocess
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any


class FastPIssueFetcher:
    def __init__(self, output_file: str = "fastp_issues.json"):
        self.base_url = "https://api.github.com/repos/OpenGene/fastp/issues"
        self.output_file = Path(output_file)

    def fetch_page(self, page: int) -> List[Dict[str, Any]]:
        """Fetch a single page of issues using curl."""
        url = f"{self.base_url}?state=all&per_page=100&page={page}&sort=created&direction=desc"
        
        cmd = [
            'curl', '-s',
            '-H', 'Accept: application/vnd.github.v3+json',
            '-H', 'User-Agent: mbf-fastq-processor-analysis',
            url
        ]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise Exception(f"curl failed: {result.stderr}")
        
        try:
            data = json.loads(result.stdout)
        except json.JSONDecodeError as e:
            raise Exception(f"Failed to parse JSON response: {e}")
        
        if isinstance(data, dict) and 'message' in data:
            raise Exception(f"GitHub API error: {data['message']}")
        
        # Filter out pull requests (GitHub treats PRs as issues)
        return [issue for issue in data if "pull_request" not in issue]

    def fetch_all_issues(self) -> List[Dict[str, Any]]:
        """Fetch all issues from the repository."""
        all_issues = []
        page = 1

        while True:
            print(f"Fetching page {page}...")
            
            issues = self.fetch_page(page)
            if not issues:
                break

            all_issues.extend(issues)
            print(f"  Found {len(issues)} issues on page {page}")

            # Check if we have more pages
            if len(issues) < 100:
                break

            page += 1
            time.sleep(0.5)  # Be nice to GitHub's API

        print(f"Total issues fetched: {len(all_issues)}")
        return all_issues

    def save_issues(self, issues: List[Dict[str, Any]]):
        """Save issues as structured JSON with metadata."""

        # Add extraction metadata
        output_data = {
            "extraction_date": datetime.now().isoformat(),
            "repository": "OpenGene/fastp",
            "total_issues": len(issues),
            "open_issues": sum(1 for issue in issues if issue["state"] == "open"),
            "closed_issues": sum(1 for issue in issues if issue["state"] == "closed"),
            "issues": issues,
        }

        # Save to JSON file
        with open(self.output_file, "w", encoding="utf-8") as f:
            json.dump(output_data, f, indent=2, ensure_ascii=False)

        print(f"Saved {len(issues)} issues to {self.output_file}")
        print(f"  - Open: {output_data['open_issues']}")
        print(f"  - Closed: {output_data['closed_issues']}")


def main():
    """Main execution function."""
    print("FastP Issues Fetcher")
    print("===================")

    fetcher = FastPIssueFetcher()

    try:
        # Fetch all issues
        issues = fetcher.fetch_all_issues()

        # Save to structured JSON
        print(f"\nSaving {len(issues)} issues to JSON...")
        fetcher.save_issues(issues)

        print("\n✅ Successfully extracted all FastP issues!")
        print(f"📁 Output: {fetcher.output_file}")

    except Exception as e:
        print(f"❌ Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    exit(main())
