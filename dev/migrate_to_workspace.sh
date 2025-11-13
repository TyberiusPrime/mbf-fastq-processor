#!/bin/bash
# Migration helper script for workspace conversion
# Usage: ./dev/migrate_to_workspace.sh [phase]

set -e

PHASE=${1:-all}

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

backup_current() {
    log_info "Creating backup of current Cargo.toml..."
    if [ ! -f Cargo.toml.original ]; then
        cp Cargo.toml Cargo.toml.original
        log_success "Backup created: Cargo.toml.original"
    else
        log_warning "Backup already exists, skipping"
    fi
}

activate_workspace() {
    log_info "Activating workspace Cargo.toml..."
    if [ -f Cargo.toml.workspace ]; then
        cp Cargo.toml.workspace Cargo.toml
        log_success "Workspace activated"
    else
        log_error "Cargo.toml.workspace not found!"
        exit 1
    fi
}

phase_core() {
    log_info "Phase 2: Migrating mbf-core..."

    # Copy reads.rs
    log_info "Copying src/io/reads.rs → crates/mbf-core/src/reads.rs"
    cp src/io/reads.rs crates/mbf-core/src/reads.rs

    # Copy dna.rs
    log_info "Copying src/dna.rs → crates/mbf-core/src/dna.rs"
    cp src/dna.rs crates/mbf-core/src/dna.rs

    log_info "Testing mbf-core compilation..."
    if cargo build -p mbf-core; then
        log_success "mbf-core compiles successfully!"
    else
        log_error "mbf-core compilation failed. Fix errors before proceeding."
        exit 1
    fi
}

phase_config() {
    log_info "Phase 3: Migrating mbf-config..."

    # Copy config directory
    log_info "Copying src/config/* → crates/mbf-config/src/"
    cp -r src/config/* crates/mbf-config/src/

    log_info "Testing mbf-config compilation..."
    if cargo build -p mbf-config; then
        log_success "mbf-config compiles successfully!"
    else
        log_error "mbf-config compilation failed. Fix errors before proceeding."
        exit 1
    fi
}

phase_io() {
    log_info "Phase 4: Migrating mbf-io..."

    # Copy I/O modules
    log_info "Copying I/O modules to crates/mbf-io/src/"
    for file in fileformats.rs input.rs output.rs parsers.rs; do
        if [ -f "src/io/$file" ]; then
            cp "src/io/$file" "crates/mbf-io/src/$file"
        fi
    done

    log_info "Testing mbf-io compilation..."
    if cargo build -p mbf-io; then
        log_success "mbf-io compiles successfully!"
    else
        log_error "mbf-io compilation failed. Fix errors before proceeding."
        exit 1
    fi
}

phase_transformations() {
    log_info "Phase 5: Migrating mbf-transformations..."

    # This phase requires manual work due to complex dependencies
    log_warning "Phase 5 requires manual migration due to complex module structure"
    log_info "See WORKSPACE_MIGRATION.md Phase 5 for detailed instructions"

    log_info "Files to migrate:"
    log_info "  - src/transformations.rs → crates/mbf-transformations/src/lib.rs"
    log_info "  - src/transformations/* → crates/mbf-transformations/src/"
    log_info "  - src/demultiplex.rs → crates/mbf-transformations/src/demultiplex.rs"
}

phase_main() {
    log_info "Phase 6: Updating main binary..."

    if [ -f Cargo.toml.main ]; then
        log_info "Activating main crate Cargo.toml..."
        # Main Cargo.toml is already the workspace root
        log_info "Main binary configuration is in workspace root"
    fi

    log_warning "Manual import updates required in:"
    log_info "  - src/lib.rs"
    log_info "  - src/main.rs"
    log_info "  - src/pipeline.rs"
    log_info "  - src/interactive.rs"

    log_info "Change: use crate::X → use mbf_X::X"
}

test_full() {
    log_info "Testing full workspace build..."
    if cargo build; then
        log_success "Full build successful!"

        log_info "Running tests..."
        if cargo test; then
            log_success "All tests pass!"
        else
            log_error "Some tests failed"
            return 1
        fi
    else
        log_error "Build failed"
        return 1
    fi
}

rollback() {
    log_warning "Rolling back to original structure..."

    if [ -f Cargo.toml.original ]; then
        cp Cargo.toml.original Cargo.toml
        log_success "Cargo.toml restored"
    fi

    log_info "To complete rollback, run: rm -rf crates/ && cargo clean"
}

show_help() {
    cat << EOF
Workspace Migration Helper

Usage: $0 [phase]

Phases:
  backup         Create backup of current Cargo.toml
  activate       Activate workspace Cargo.toml
  core           Migrate mbf-core (Phase 2)
  config         Migrate mbf-config (Phase 3)
  io             Migrate mbf-io (Phase 4)
  transformations Migrate mbf-transformations (Phase 5)
  main           Update main binary (Phase 6)
  test           Test full build and tests
  all            Run all phases sequentially
  rollback       Restore original structure
  help           Show this help message

Example:
  $0 backup
  $0 activate
  $0 core
  $0 test

EOF
}

case $PHASE in
    backup)
        backup_current
        ;;
    activate)
        backup_current
        activate_workspace
        ;;
    core)
        phase_core
        ;;
    config)
        phase_config
        ;;
    io)
        phase_io
        ;;
    transformations)
        phase_transformations
        ;;
    main)
        phase_main
        ;;
    test)
        test_full
        ;;
    all)
        backup_current
        activate_workspace
        phase_core
        phase_config
        phase_io
        phase_transformations
        phase_main
        log_info "Migration phases complete. Manual fixes may be required."
        log_info "Run: $0 test"
        ;;
    rollback)
        rollback
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "Unknown phase: $PHASE"
        show_help
        exit 1
        ;;
esac
