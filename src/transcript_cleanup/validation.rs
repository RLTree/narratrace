#[derive(Debug, Clone)]
struct CleanupValidation {
    status: String,
    reason: String,
}

impl CleanupValidation {
    fn is_verified(&self) -> bool {
        self.status == "verified-conservative-transform"
    }
}

fn validate_cleanup_output(raw: &str, cleaned: &str, dictionary: &[String]) -> CleanupValidation {
    let raw_tokens = normalized_tokens(raw);
    let cleaned_tokens = normalized_tokens(cleaned);
    if raw_tokens.is_empty() || cleaned_tokens.is_empty() {
        return untrusted("empty transcript or cleanup output");
    }
    if raw_tokens.len() > 100_000 || cleaned_tokens.len() > 100_000 {
        return untrusted("cleanup validation token bound exceeded");
    }
    let allowed = dictionary
        .iter()
        .map(|entry| normalized_tokens(entry).concat())
        .collect::<std::collections::BTreeSet<_>>();
    let mut reachable = std::collections::BTreeSet::from([(0_usize, 0_usize)]);
    while let Some((raw_at, cleaned_at)) = reachable.pop_first() {
        if raw_at == raw_tokens.len() && cleaned_at == cleaned_tokens.len() {
            return CleanupValidation {
                status: "verified-conservative-transform".to_string(),
                reason: "token order and semantic content preserved".to_string(),
            };
        }
        for raw_count in 1..=4 {
            for cleaned_count in 1..=4 {
                if raw_at + raw_count > raw_tokens.len()
                    || cleaned_at + cleaned_count > cleaned_tokens.len()
                {
                    continue;
                }
                let before = raw_tokens[raw_at..raw_at + raw_count].concat();
                let after = cleaned_tokens[cleaned_at..cleaned_at + cleaned_count].concat();
                let dictionary_correction =
                    allowed.contains(&after) && bounded_edit_distance(&before, &after, 2).is_some();
                let minor_correction = raw_count == 1
                    && cleaned_count == 1
                    && before.len() >= 4
                    && bounded_edit_distance(&before, &after, 1).is_some();
                if before == after || dictionary_correction || minor_correction {
                    reachable.insert((raw_at + raw_count, cleaned_at + cleaned_count));
                }
            }
        }
    }
    untrusted("cleanup output added, removed, reordered, or materially changed words")
}

fn normalized_tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_lowercase())
        .collect()
}

fn bounded_edit_distance(left: &str, right: &str, limit: usize) -> Option<usize> {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    if left.len().abs_diff(right.len()) > limit {
        return None;
    }
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (row, left_ch) in left.iter().enumerate() {
        let mut current = vec![row + 1];
        for (column, right_ch) in right.iter().enumerate() {
            current.push(std::cmp::min(
                std::cmp::min(current[column] + 1, previous[column + 1] + 1),
                previous[column] + usize::from(left_ch != right_ch),
            ));
        }
        previous = current;
    }
    (previous[right.len()] <= limit).then_some(previous[right.len()])
}

fn untrusted(reason: &str) -> CleanupValidation {
    CleanupValidation {
        status: "untrusted-model-output".to_string(),
        reason: reason.to_string(),
    }
}
