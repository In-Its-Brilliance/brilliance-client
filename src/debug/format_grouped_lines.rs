use std::time::Duration;

const PARENTS_LIMIT: usize = 10;
const CHILDREN_LIMIT: usize = 5;

const NAME_WIDTH_PARENT: usize = 30;
const NAME_WIDTH_CHILD: usize = 30;
const TIME_WIDTH: usize = 4;

fn fmt_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn cut(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

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
            "  - &a{name:<NAME_WIDTH_PARENT$}&r - &a{last:>TIME_WIDTH$.1}ms&r &7(avg {avg:.1}ms)&r",
            name=cut(root, NAME_WIDTH_PARENT),
            last=fmt_ms(parent_last),
            avg=fmt_ms(parent_avg),
        ));

        let mut children: Vec<_> = parts.iter().filter(|(name, _, _)| *name != root).collect();

        children.sort_by(|a, b| b.1.cmp(&a.1));

        for (name, last, avg) in children.into_iter().take(CHILDREN_LIMIT) {
            let percent = if parent_last.as_nanos() > 0 {
                last.as_secs_f64() / parent_last.as_secs_f64() * 100.0
            } else {
                0.0
            };

            lines.push('\n');
            lines.push_str(&format!(
                "    > &e{name:<NAME_WIDTH_CHILD$}&r > &8{last:>TIME_WIDTH$.1}ms&r {percent:>2.0}%&r &7(avg {avg:.1}ms)&r",
                name=cut(name.split("::").last().unwrap(), NAME_WIDTH_CHILD),
                last=fmt_ms(*last),
                percent=percent,
                avg=fmt_ms(*avg),
            ));
        }

        if i + 1 < PARENTS_LIMIT {
            lines.push('\n');
        }
    }

    lines
}
