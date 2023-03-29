pub mod utils {
    use lazy_static::lazy_static;
    use std::{
        fs::File,
        io::{self, BufRead, BufReader},
        path::Path,
    };

    use csv::Writer;
    use regex::Regex;

    lazy_static! {
        static ref TODO_PATTERN: Regex =
            Regex::new(r"(?m)^(?:\s*//|\s*#)\s*TODO:\s*(.*\S)\s*$").unwrap();
    }

    #[derive(Debug, PartialEq, Eq)]
    enum FileExtension {
        Rust,
        Python,
        Java,
        TypeScript,
        JavaScript,
    }

    impl FileExtension {
        fn from_str(ext: &str) -> Option<Self> {
            match ext {
                "rs" => Some(Self::Rust),
                "py" => Some(Self::Python),
                "java" => Some(Self::Java),
                "ts" => Some(Self::TypeScript),
                "js" => Some(Self::JavaScript),
                _ => None,
            }
        }
    }

    /// Extracts a single-line TODO comment from the given line of Rust or Python source code.
    ///
    /// This function takes a line of source code and a reference to a compiled regular expression
    /// matching TODO comments in Rust and Python code. If a TODO comment is found,
    /// the function returns the comment text as a `String` wrapped in `Some`.
    /// If no TODO comment is found, the function returns `None`.
    ///
    /// # Arguments
    ///
    /// * `line` - A reference to a string containing the line of source code.
    /// * `todo_pattern` - A reference to a compiled regular expression that matches
    ///                    single-line TODO comments in Rust and Python source code.
    pub fn extract_todo_comment(line: &str, todo_pattern: &Regex) -> Option<String> {
        if let Some(captures) = todo_pattern.captures(line) {
            Some(
                captures
                    .get(1)
                    .map_or(String::new(), |m| m.as_str().to_string()),
            )
        } else {
            None
        }
    }

    pub fn is_supported_file(entry: &ignore::DirEntry) -> bool {
        entry.path().is_file()
            && entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str().and_then(|s| FileExtension::from_str(s)))
                .is_some()
    }

    pub fn process_file(path: &Path, csv_writer: &mut Writer<std::fs::File>) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for (line_number, line) in reader.lines().enumerate() {
            let line = line?;
            if let Some(todo_comment) = extract_todo_comment(&line, &*TODO_PATTERN) {
                csv_writer.write_record(&[
                    path.to_str().unwrap_or_default(),
                    &(line_number + 1).to_string(),
                    &todo_comment,
                ])?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::utils;
    use lazy_static::lazy_static;
    use regex::Regex;

    lazy_static! {
        static ref TODO_PATTERN: Regex =
            Regex::new(r"(?m)^(?:\s*//|\s*#)\s*TODO:\s*(.*\S)\s*$").unwrap();
    }

    #[test]
    fn test_extract_todo_comment() {
        let line_with_todo = "    // TODO: Implement the new feature";
        let line_without_todo = "    // This is a regular comment";
        let line_with_no_comment = "let x = 5;";

        assert_eq!(
            utils::extract_todo_comment(line_with_todo, &*TODO_PATTERN),
            Some(String::from("Implement the new feature"))
        );
        assert_eq!(
            utils::extract_todo_comment(line_without_todo, &*TODO_PATTERN),
            None
        );
        assert_eq!(
            utils::extract_todo_comment(line_with_no_comment, &*TODO_PATTERN),
            None
        );
    }

    #[test]
    fn test_extract_todo_comment_with_whitespace() {
        let line_with_todo = "    //   TODO:  Improve error handling  ";
        let expected = Some(String::from("Improve error handling"));

        assert_eq!(
            utils::extract_todo_comment(line_with_todo, &*TODO_PATTERN),
            expected
        );
    }

    #[test]
    fn test_extract_todo_comment_multiline() {
        let line_with_multiline_todo =
            "    // TODO: Refactor this code\n    // to make it more efficient";
        let expected = Some(String::from("Refactor this code"));

        assert_eq!(
            utils::extract_todo_comment(line_with_multiline_todo, &*TODO_PATTERN),
            expected
        );
    }

    #[test]
    fn test_extract_todo_comment_inline() {
        let line_with_inline_todo =
            "let x = 5; // TODO: Use a constant instead of a hardcoded value";
        let expected = None;

        assert_eq!(
            utils::extract_todo_comment(line_with_inline_todo, &*TODO_PATTERN),
            expected
        );
    }

    #[test]
    fn test_extract_todo_comment_python() {
        let line_with_todo = "# TODO: Implement the new feature";
        let line_without_todo = "# This is a regular comment";
        let line_with_no_comment = "x = 5";

        assert_eq!(
            utils::extract_todo_comment(line_with_todo, &*TODO_PATTERN),
            Some(String::from("Implement the new feature"))
        );
        assert_eq!(
            utils::extract_todo_comment(line_without_todo, &*TODO_PATTERN),
            None
        );
        assert_eq!(
            utils::extract_todo_comment(line_with_no_comment, &*TODO_PATTERN),
            None
        );
    }

    #[test]
    fn test_extract_todo_comment_python_inline() {
        let line_with_inline_todo = "x = 5  # TODO: Use a constant instead of a hardcoded value";
        let expected = None;

        assert_eq!(
            utils::extract_todo_comment(line_with_inline_todo, &*TODO_PATTERN),
            expected
        );
    }
}
