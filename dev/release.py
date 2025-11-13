#!/usr/bin/env python3
"""
Release script for mbf-fastq-processor

This script:
1. Verifies there are no uncommitted changes
2. Reads current version from Cargo.toml
3. Asks user for release type (patch/minor/major)
4. Updates version in Cargo.toml
5. Commits changes with jj
6. Creates bookmark and git tag
7. Pushes to remote with tags
"""

import subprocess
import sys
import re
import toml
from pathlib import Path


def run_command(cmd, check=True, capture_output=True):
    """Run a command and return the result"""
    try:
        result = subprocess.run(
            cmd, shell=True, check=check, capture_output=capture_output, text=True
        )
        return result.stdout.strip() if capture_output else None
    except subprocess.CalledProcessError as e:
        if capture_output:
            print(f"Command failed: {cmd}")
            print(f"Error: {e.stderr}")
        raise


def check_uncommitted_changes():
    """Check if there are any uncommitted changes"""
    print("Checking for uncommitted changes...")

    # Check jj status
    status = run_command("jj status")

    # If status is empty or only shows "The working copy is clean", we're good
    if not status or "The working copy has no changes" in status:
        print("✓ Working directory is clean")
        return

    print("❌ There are uncommitted changes:")
    print(status)
    print("\nPlease commit your changes before creating a release.")
    sys.exit(1)


def get_current_version():
    """Read current version from Cargo.toml"""
    cargo_toml_path = Path("Cargo.toml")
    if not cargo_toml_path.exists():
        print("❌ Cargo.toml not found!")
        sys.exit(1)

    with open(cargo_toml_path, "r") as f:
        cargo_data = toml.load(f)

    current_version = cargo_data["package"]["version"]
    print(f"Current version: {current_version}")
    return current_version


def parse_version(version_str):
    """Parse semantic version string into major.minor.patch"""
    match = re.match(r"^(\d+)\.(\d+)\.(\d+)$", version_str)
    if not match:
        print(f"❌ Invalid version format: {version_str}")
        sys.exit(1)

    return int(match.group(1)), int(match.group(2)), int(match.group(3))


def get_release_type():
    """Ask user for release type"""
    print("\nWhat type of release is this?")
    print("1. Patch release (bug fixes, no new features)")
    print("2. Minor release (new features, backward compatible)")
    print("3. Major release (breaking changes)")

    while True:
        choice = input("\nEnter your choice (1/2/3): ").strip()
        if choice in ["1", "2", "3"]:
            return ["patch", "minor", "major"][int(choice) - 1]
        print("Please enter 1, 2, or 3")


def bump_version(current_version, release_type):
    """Bump version according to release type"""
    major, minor, patch = parse_version(current_version)

    if release_type == "patch":
        patch += 1
    elif release_type == "minor":
        minor += 1
        patch = 0
    elif release_type == "major":
        major += 1
        minor = 0
        patch = 0

    new_version = f"{major}.{minor}.{patch}"
    print(f"New version will be: {new_version}")
    return new_version


def update_cargo_toml(new_version):
    """Update version in Cargo.toml"""
    print(f"Updating Cargo.toml to version {new_version}...")

    cargo_toml_path = Path("Cargo.toml")
    with open(cargo_toml_path, "r") as f:
        content = f.read()

    # Replace the version line
    pattern = r'^version = "[^"]*"'
    replacement = f'version = "{new_version}"'
    new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE)

    with open(cargo_toml_path, "w") as f:
        f.write(new_content)

    print("✓ Cargo.toml updated")


def confirm_release(new_version):
    """Ask for final confirmation"""
    print(f"\n🚀 Ready to release version {new_version}")
    print("This will:")
    print(f"  - Commit changes to Cargo.toml")
    print(f"  - Create bookmark v{new_version}")
    print(f"  - Create git tag v{new_version}")
    print(f"  - Push changes and tags to remote")

    while True:
        confirm = input("\nProceed with release? (y/n): ").strip().lower()
        if confirm in ["y", "yes"]:
            return True
        elif confirm in ["n", "no"]:
            print("Release cancelled.")
            return False
        print("Please enter 'y' or 'n'")


def create_release(new_version):
    """Create the release: commit, bookmark, tag, and push"""
    print(f"\nCreating release v{new_version}...")

    # Commit changes
    print("Committing changes...")
    run_command(f'jj commit -m "Release v{new_version}"', capture_output=False)

    # Create bookmark
    print(f"Creating bookmark v{new_version}...")
    run_command(f"jj bookmark set v{new_version}", capture_output=False)

    print("and moving main")
    run_command("jj bookmark set main", capture_output=False)

    # Create git tag
    print(f"Creating git tag v{new_version}...")
    run_command(f"git tag v{new_version}", capture_output=False)

    # Push changes
    print("Pushing changes...")
    run_command("jj git push --allow-new", capture_output=False)

    # Push tags
    print("Pushing tags...")
    run_command("git push --tags", capture_output=False)

    print(f"✅ Release v{new_version} created successfully!")


def main():
    print("🚀 mbf-fastq-processor Release Script")
    print("=====================================")

    try:
        # Step 1: Check for uncommitted changes
        check_uncommitted_changes()

        # Step 2: Get current version
        current_version = get_current_version()

        # Step 3: Ask for release type
        release_type = get_release_type()

        # Step 4: Calculate new version
        new_version = bump_version(current_version, release_type)

        # Step 5: Update Cargo.toml
        update_cargo_toml(new_version)

        # Step 6: Confirm and create release
        if confirm_release(new_version):
            create_release(new_version)
        else:
            # Restore original Cargo.toml
            run_command("jj restore Cargo.toml")
            print("Cargo.toml restored to original state.")

    except KeyboardInterrupt:
        print("\n\nRelease cancelled by user.")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ Error during release: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
