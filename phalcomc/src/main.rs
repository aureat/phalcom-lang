use anyhow::Result;
use phalcom_ast::parse;
use std::{fs, path::Path};

fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: phalcomc <file.phalcom>");
    let source = fs::read_to_string(Path::new(&path))?;

    let program = parse(source.as_str(), 0).map_err(|errors| {
        for (_error, range) in errors {
            // Get error byte range
            let start = range.start;
            let line_start = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line_end = source[start..]
                .find('\n')
                .map(|i| start + i)
                .unwrap_or(source.len());

            let line_text = &source[line_start..line_end];
            let line_number = source[..start].matches('\n').count() + 1;
            let col = start - line_start;

            eprintln!("{_error}");
            eprintln!(
                "At line {} column {}",
                line_number,
                col + 1,
            );
            eprintln!("  {}", line_text);
            eprintln!("  {}^", " ".repeat(col));
        }
        anyhow::anyhow!("Parsing failed with errors")
    })?;

    println!("{:#?}", program);
    Ok(())
}
