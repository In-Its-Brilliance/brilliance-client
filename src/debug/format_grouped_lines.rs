use std::time::Duration;

const PARENTS_LIMIT: usize = 10;
const CHILDREN_LIMIT: usize = 5;

pub(crate) fn format_grouped_lines(items: Vec<(&'static str, Duration, Duration)>) -> String {
    use std::collections::HashMap;

    let mut grouped: HashMap<&str, Vec<(&'static str, Duration, Duration)>> = HashMap::new();

    for (name, last, avg) in items {
        let root = name.split("::").next().unwrap();
        grouped.entry(root).or_default().push((name, last, avg));
    }

    let mut roots: Vec<(&str, Duration)> = grouped
        .iter()
        .map(|(root, v)| {
            let parent_last = v
                .iter()
                .find(|(name, _, _)| *name == *root)
                .map(|(_, last, _)| *last)
                .unwrap_or(Duration::ZERO);

            (*root, parent_last)
        })
        .collect();

    roots.sort_by(|a, b| b.1.cmp(&a.1));

    let mut lines = String::new();

    for (i, (root, parent_last)) in roots.into_iter().take(PARENTS_LIMIT).enumerate() {
        let parts = &grouped[root];

        let parent_avg = parts
            .iter()
            .find(|(name, _, _)| *name == root)
            .map(|(_, _, avg)| *avg)
            .unwrap_or(Duration::ZERO);

        lines.push_str(&format!(
            "  - &a{}&r &8{:.1?} &7(avg {:.1?})",
            root, parent_last, parent_avg
        ));

        let mut children: Vec<_> = parts
            .iter()
            .filter(|(name, _, _)| *name != root)
            .collect();

        children.sort_by(|a, b| b.1.cmp(&a.1));

        for (name, last, avg) in children.into_iter().take(CHILDREN_LIMIT) {
            let percent = if parent_last.as_nanos() > 0 {
                last.as_secs_f64() / parent_last.as_secs_f64() * 100.0
            } else {
                0.0
            };

            lines.push('\n');
            lines.push_str(&format!(
                "      > &e{}&r &8{:.1?} {:.0}% &7(avg {:.1?})",
                name.split("::").last().unwrap(),
                last,
                percent,
                avg
            ));
        }

        if i + 1 < PARENTS_LIMIT {
            lines.push('\n');
        }
    }

    lines
}
