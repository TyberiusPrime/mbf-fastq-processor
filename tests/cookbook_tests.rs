// This file is written by dev/update_cookbook_tests.py
// Cookbook tests verify that example cookbooks run successfully.
// They are run separately from regular tests (only in CI/nix builds).

#[cfg(feature = "cookbook_tests")]
mod test_runner;

#[cfg(feature = "cookbook_tests")]
use test_runner::run_test;

#[test]
#[cfg(feature = "cookbook_tests")]
fn cookbook_01_basic_quality_report() {
    println!("Testing cookbook: cookbooks/01-basic-quality-report");
    run_test(std::path::Path::new("cookbooks/01-basic-quality-report"));
}

#[test]
#[cfg(feature = "cookbook_tests")]
fn cookbook_02_umi_extraction() {
    println!("Testing cookbook: cookbooks/02-umi-extraction");
    run_test(std::path::Path::new("cookbooks/02-umi-extraction"));
}

#[test]
#[cfg(feature = "cookbook_tests")]
fn cookbook_03_lexogen_quantseq() {
    println!("Testing cookbook: cookbooks/03-lexogen-quantseq");
    run_test(std::path::Path::new("cookbooks/03-lexogen-quantseq"));
}
