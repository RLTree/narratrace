include!("review/build.rs");
include!("review/transcript_quality.rs");
include!("review/proof.rs");
include!("review/artifact.rs");
include!("review/helpers.rs");

#[cfg(test)]
mod tests;

#[cfg(test)]
mod product_cohesion_tests;

#[cfg(test)]
mod proof_tests;
