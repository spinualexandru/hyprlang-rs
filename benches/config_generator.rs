//! Generates synthetic hyprlang configs of specified line counts for benchmarking

pub fn generate_config(target_lines: usize) -> String {
    let mut output = String::with_capacity(target_lines * 50);

    // Header with variables
    output.push_str("# Synthetic benchmark config\n");
    output.push_str("$SCALE = 2\n");
    output.push_str("$WIDTH = 1920\n");
    output.push_str("$HEIGHT = 1080\n");
    output.push('\n');

    let mut lines = 5;
    let mut category_num = 0;
    let mut in_category = false;

    while lines < target_lines {
        // Start a new category every ~25 lines
        if !in_category {
            output.push_str(&format!("category{} {{\n", category_num));
            category_num += 1;
            in_category = true;
            lines += 1;
            if lines >= target_lines {
                break;
            }
        }

        // Add values inside category
        let values_in_category = (target_lines - lines).clamp(1, 23);
        for i in 0..values_in_category {
            if lines >= target_lines - 1 {
                break;
            }
            let val_id = (category_num - 1) * 25 + i;
            match i % 6 {
                0 => output.push_str(&format!("    int_{} = {}\n", val_id, val_id * 10)),
                1 => output.push_str(&format!("    float_{} = {:.2}\n", val_id, val_id as f64 * 0.5)),
                2 => output.push_str(&format!("    str_{} = value_{}\n", val_id, val_id)),
                3 => output.push_str(&format!(
                    "    color_{} = rgba({:02x}{:02x}{:02x}ff)\n",
                    val_id,
                    val_id % 256,
                    (val_id * 2) % 256,
                    (val_id * 3) % 256
                )),
                4 => output.push_str(&format!("    vec_{} = ({}, {})\n", val_id, val_id, val_id * 2)),
                5 => output.push_str(&format!("    bool_{} = {}\n", val_id, val_id % 2 == 0)),
                _ => unreachable!(),
            }
            lines += 1;
        }

        // Close category
        output.push_str("}\n");
        lines += 1;
        in_category = false;

        // Add blank line between categories
        if lines < target_lines - 2 {
            output.push('\n');
            lines += 1;
        }
    }

    // Ensure we close any open category
    if in_category {
        output.push_str("}\n");
    }

    output
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::generate_config;

    #[test]
    fn test_generate_small() {
        let config = generate_config(50);
        let lines = config.lines().count();
        assert!((48..=52).contains(&lines), "Got {} lines", lines);
    }

    #[test]
    fn test_generate_large() {
        let config = generate_config(1000);
        let lines = config.lines().count();
        assert!((998..=1002).contains(&lines), "Got {} lines", lines);
    }

    #[test]
    fn test_parseable() {
        let config = generate_config(100);
        assert!(config.contains("category"));
        assert!(config.contains("}"));
        assert!(!config.ends_with("{\n"));
    }
}
