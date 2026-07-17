use crate::config::parse_args_from;

#[test]
fn coverage_receipt_defaults_to_authorized_skill_proof_root() {
    let args = parse_args_from([
        "nrr",
        "coverage-receipt",
        "--skill-dir",
        "/private/tmp/narrated-record-replay/skill",
    ])
    .unwrap();
    assert!(args.coverage_receipt.is_none());
}

#[test]
fn custom_coverage_receipt_requires_explicit_path_consent() {
    let input = [
        "nrr",
        "coverage-receipt",
        "--skill-dir",
        "/private/tmp/narrated-record-replay/skill",
        "--coverage-receipt",
        "/private/tmp/narrated-record-replay/other/receipt.json",
    ];
    assert!(parse_args_from(input).is_err());
    let mut consented = input.to_vec();
    consented.push("--i-consent-to-custom-runtime-paths");
    assert!(parse_args_from(consented).is_ok());
}
