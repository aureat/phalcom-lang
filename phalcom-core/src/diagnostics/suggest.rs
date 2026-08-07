//! did-you-mean suggest engine (IS §9).

use std::cmp::Ordering;

/// The category of edit required to transform the string.
/// Lower values are preferred (ranked higher).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum EditCategory {
    /// The strings differ only in case.
    CaseInsensitiveEqual = 0,
    /// The string difference consists only of transpositions.
    Transposition = 1,
    /// The string difference includes substitutions but no insertions/deletions.
    Substitution = 2,
    /// The string difference includes insertions and/or deletions.
    InsDel = 3,
}

/// Calculates the Optimal String Alignment (restricted Damerau-Levenshtein) distance
/// and the associated edit category between two strings.
pub fn os_distance(a: &str, b: &str) -> (u32, EditCategory) {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    // d[i][j] stores (distance, category)
    let mut d = vec![vec![(0u32, EditCategory::Transposition); n + 1]; m + 1];

    for (i, row) in d.iter_mut().enumerate() {
        row[0] = (i as u32, if i == 0 { EditCategory::Transposition } else { EditCategory::InsDel });
    }
    for (j, entry) in d[0].iter_mut().enumerate() {
        *entry = (j as u32, if j == 0 { EditCategory::Transposition } else { EditCategory::InsDel });
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };

            // Deletion
            let (dist_del, cat_del) = d[i - 1][j];
            let opt_del = (dist_del + 1, std::cmp::max(cat_del, EditCategory::InsDel));

            // Insertion
            let (dist_ins, cat_ins) = d[i][j - 1];
            let opt_ins = (dist_ins + 1, std::cmp::max(cat_ins, EditCategory::InsDel));

            // Substitution
            let (dist_sub, cat_sub) = d[i - 1][j - 1];
            let opt_sub = if cost == 0 {
                (dist_sub, cat_sub)
            } else {
                (dist_sub + 1, std::cmp::max(cat_sub, EditCategory::Substitution))
            };

            let mut best = std::cmp::min(opt_del, opt_ins);
            best = std::cmp::min(best, opt_sub);

            // Transposition
            if i > 1 && j > 1 && a_chars[i - 1] == b_chars[j - 2] && a_chars[i - 2] == b_chars[j - 1] {
                let (dist_trans, cat_trans) = d[i - 2][j - 2];
                let opt_trans = (dist_trans + 1, std::cmp::max(cat_trans, EditCategory::Transposition));
                best = std::cmp::min(best, opt_trans);
            }

            d[i][j] = best;
        }
    }

    let (dist, mut cat) = d[m][n];
    if a.eq_ignore_ascii_case(b) {
        cat = EditCategory::CaseInsensitiveEqual;
    }
    (dist, cat)
}

/// Finds the best match for `miss` among the provided `candidates` using Damerau-Levenshtein OSA.
///
/// Matches are filtered by thresholds based on the length of `miss`:
/// - len <= 4: max distance 1
/// - len 5-8: max distance 2
/// - len > 8: max distance 3
///
/// Ranking priority:
/// 1. Edit Category (CaseInsensitiveEqual > Transposition > Substitution > InsDel)
/// 2. Distance (smaller is better)
/// 3. Shorter candidate length
/// 4. Lexicographic order
///
/// Returns `None` if the best candidate is not strictly better than the runner-up.
pub fn best_match<'a>(miss: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let miss_len = miss.chars().count();
    if miss_len == 0 {
        return None;
    }

    let max_dist = if miss_len <= 4 {
        1
    } else if miss_len <= 8 {
        2
    } else {
        3
    };

    struct MatchCandidate<'a> {
        name: &'a str,
        category: EditCategory,
        distance: u32,
    }

    let mut matches = Vec::new();

    for candidate in candidates {
        let (dist, cat) = os_distance(miss, candidate);
        if dist <= max_dist {
            matches.push(MatchCandidate {
                name: candidate,
                category: cat,
                distance: dist,
            });
        }
    }

    if matches.is_empty() {
        return None;
    }

    // Sort according to ranking requirements
    matches.sort_by(|x, y| {
        let cat_cmp = x.category.cmp(&y.category);
        if cat_cmp != Ordering::Equal {
            return cat_cmp;
        }

        let dist_cmp = x.distance.cmp(&y.distance);
        if dist_cmp != Ordering::Equal {
            return dist_cmp;
        }

        let len_cmp = x.name.chars().count().cmp(&y.name.chars().count());
        if len_cmp != Ordering::Equal {
            return len_cmp;
        }

        x.name.cmp(y.name)
    });

    if matches.len() > 1 {
        let best = &matches[0];
        let runner_up = &matches[1];
        if best.category == runner_up.category && best.distance == runner_up.distance && best.name.chars().count() == runner_up.name.chars().count() {
            // Tie-suppression: same category, distance, and length — no confidence.
            return None;
        }
    }

    Some(matches[0].name)
}

/// Suggests a selector, taking into account arity mismatches for exact base name matches.
pub fn suggest_selector(miss_selector: &str, candidates: impl Iterator<Item = String>) -> Option<String> {
    let (miss_base, miss_labels, _) = crate::method::decode_selector(miss_selector);
    let miss_arity = miss_labels.len();

    let mut exact_base_matches = Vec::new();
    let mut all_cands = Vec::new();

    for cand in candidates {
        let (cand_base, cand_labels, _) = crate::method::decode_selector(&cand);
        if cand_base == miss_base {
            exact_base_matches.push((cand, cand_labels.len()));
        } else {
            all_cands.push(cand);
        }
    }

    if !exact_base_matches.is_empty() {
        // Find the one closest in arity
        exact_base_matches.sort_by_key(|&(_, arity)| (arity as isize - miss_arity as isize).abs());
        let (best_cand, best_arity) = &exact_base_matches[0];
        let arg_str = if *best_arity == 1 {
            "1 argument"
        } else {
            &format!("{} arguments", best_arity)
        };
        return Some(format!("'{}' exists — did you mean to pass {}?", best_cand, arg_str));
    }

    // Otherwise, fall back to best_match
    let cand_refs: Vec<&str> = all_cands.iter().map(|s| s.as_str()).collect();
    best_match(miss_selector, cand_refs.into_iter()).map(|sug| format!("did you mean '{}'?", sug))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_distance() {
        // Transposition
        assert_eq!(os_distance("ab", "ba"), (1, EditCategory::Transposition));
        // Substitution
        assert_eq!(os_distance("ab", "ac"), (1, EditCategory::Substitution));
        // Insertion
        assert_eq!(os_distance("ab", "abc"), (1, EditCategory::InsDel));
        // Deletion
        assert_eq!(os_distance("ab", "a"), (1, EditCategory::InsDel));
        // Case-insensitive equal
        assert_eq!(os_distance("Ab", "aB"), (2, EditCategory::CaseInsensitiveEqual));
    }

    #[test]
    fn test_best_match_thresholds() {
        let candidates = vec!["negated", "negate", "negatd_other"];

        // len <= 4: max distance 1
        assert_eq!(best_match("neg", candidates.iter().copied()), None);

        // len 5..8: max distance 2
        assert_eq!(best_match("negatd", candidates.iter().copied()), Some("negate"));
        assert_eq!(best_match("negatdd", candidates.iter().copied()), Some("negated"));
    }

    #[test]
    fn test_best_match_tie_breaking_and_determinism() {
        // Same category and distance: tie-suppression (returns None)
        let candidates = vec!["abc", "abd"];
        assert_eq!(best_match("abe", candidates.iter().copied()), None);

        // Different category / distance -> choose best
        let candidates2 = vec!["ab", "abe"];
        // "ab" is distance 1 (InsDel), "abe" is distance 1 (Substitution).
        // Substitution category (2) is preferred over InsDel (3).
        assert_eq!(best_match("abc", candidates2.iter().copied()), Some("abe"));
    }
}
