include!("packet/markdown.rs");
include!("packet/build.rs");
include!("packet/evidence_and_summary.rs");
#[cfg(test)]
mod markdown_tests;
#[cfg(test)]
mod material_tests;
#[cfg(test)]
mod security_tests;
include!("packet/tests.rs");
