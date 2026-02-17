// ==============================================================================
// String Similarity Utilities
// ==============================================================================
//
// Shared edit-distance helpers used by both the reader (keyword typo detection)
// and the compiler ("did you mean?" suggestions for undefined type names).

/// Compute the Levenshtein edit distance between two strings.
///
/// Uses the standard dynamic programming algorithm with a two-row buffer
/// (O(min(m, n)) space). This is sufficient for identifiers and type names,
/// which are short.
pub(crate) fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use a single-row DP approach for space efficiency. We maintain two rows
    // and swap them after processing each character of `a`.
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr_row[j + 1] = (prev_row[j] + cost) // substitution
                .min(prev_row[j + 1] + 1) // deletion
                .min(curr_row[j] + 1); // insertion
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }
    prev_row[b_len]
}

/// Maximum edit distance for a suggestion to be considered "close enough."
///
/// For short names (length <= 4), we require distance <= 1 to avoid noisy
/// suggestions. For longer names, we allow distance <= 2.
pub(crate) fn max_edit_distance(name_len: usize) -> usize {
    if name_len <= 4 { 1 } else { 2 }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Levenshtein edit distance
    // =========================================================================

    #[test]
    fn identical_strings() {
        assert_eq!(levenshtein("record", "record"), 0);
        assert_eq!(levenshtein("string", "string"), 0);
    }

    #[test]
    fn empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "xyz"), 3);
    }

    #[test]
    fn single_substitution() {
        assert_eq!(levenshtein("string", "strang"), 1);
    }

    #[test]
    fn single_insertion() {
        assert_eq!(levenshtein("sting", "string"), 1);
        assert_eq!(levenshtein("protocoll", "protocol"), 1);
    }

    #[test]
    fn single_deletion() {
        assert_eq!(levenshtein("string", "sting"), 1);
        assert_eq!(levenshtein("protcol", "protocol"), 1);
    }

    #[test]
    fn transposition_counts_as_two_edits() {
        // Swapping adjacent characters requires a deletion + insertion.
        assert_eq!(levenshtein("recrod", "record"), 2);
    }

    #[test]
    fn two_edits() {
        assert_eq!(levenshtein("dubble", "double"), 2);
    }

    #[test]
    fn insertion_not_two_edits() {
        // These look like they might be two edits but are actually one.
        assert_eq!(levenshtein("stiring", "string"), 1);
        assert_eq!(levenshtein("bolean", "boolean"), 1);
    }

    #[test]
    fn case_difference() {
        assert_eq!(levenshtein("String", "string"), 1);
        assert_eq!(levenshtein("INT", "int"), 3);
    }

    // =========================================================================
    // max_edit_distance threshold
    // =========================================================================

    #[test]
    fn short_names_allow_distance_one() {
        assert_eq!(max_edit_distance(1), 1);
        assert_eq!(max_edit_distance(3), 1);
        assert_eq!(max_edit_distance(4), 1);
    }

    #[test]
    fn longer_names_allow_distance_two() {
        assert_eq!(max_edit_distance(5), 2);
        assert_eq!(max_edit_distance(10), 2);
    }
}
