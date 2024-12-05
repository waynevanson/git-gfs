use anyhow::{anyhow, Result};
use clap::Parser;
use gix::{attrs::glob::Pattern, glob::parse};

#[derive(Parser)]
pub struct Track {
    #[arg(value_parser = pattern_from_str)]
    pattern: Pattern,
}

fn pattern_from_str(str: &str) -> Result<Pattern> {
    let pattern = parse(str).ok_or_else(|| anyhow!("Expected a pattern but received '{}'", str))?;
    Ok(pattern)
}

#[cfg(test)]
mod test {
    use super::pattern_from_str;
    use gix::glob::{pattern::Mode, Pattern};

    #[test]
    fn parse_pattern_from_str() {
        let pattern = "*.txt";
        let result = pattern_from_str(pattern).unwrap();

        let expected = Pattern {
            text: pattern.into(),
            first_wildcard_pos: Some(0),
            mode: Mode::NO_SUB_DIR | Mode::ENDS_WITH,
        };

        assert_eq!(result, expected);
    }
}
