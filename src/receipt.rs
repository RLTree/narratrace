include!("receipt/write.rs");
include!("receipt/evidence.rs");
include!("receipt/verification.rs");

#[cfg(test)]
mod tests;

#[cfg(test)]
mod write_tests;

#[cfg(test)]
mod verification_tests;

#[cfg(test)]
mod artifact_tests;

#[cfg(test)]
mod security_binding_tests;
