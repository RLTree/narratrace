include!("transcript_alignment/types_and_io.rs");
include!("transcript_alignment/authority.rs");
include!("transcript_alignment/limits.rs");
include!("transcript_alignment/alignment_windows.rs");
include!("transcript_alignment/tokens_and_spans.rs");
include!("transcript_alignment/similarity_and_output.rs");

#[cfg(test)]
mod tests {
    include!("transcript_alignment/tests_part1.rs");
    include!("transcript_alignment/tests_part2.rs");
    include!("transcript_alignment/io_tests.rs");
    include!("transcript_alignment/window_tests.rs");
    include!("transcript_alignment/limit_tests.rs");
}
