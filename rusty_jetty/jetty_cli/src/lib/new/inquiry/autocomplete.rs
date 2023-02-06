use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context;
use inquire::{autocompletion::Replacement, Autocomplete, CustomUserError};

#[derive(Debug, Clone, Default)]
pub(crate) struct FilepathCompleter {
    input: String,
    paths: Vec<String>,
    lcp: String,
}

impl FilepathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        self.paths.clear();

        let input_path = std::path::PathBuf::from(input)
            .expand_tilde()
            .context("can't expand tilde")?;

        self.input = input_path.to_string_lossy().into_owned();

        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    std::path::PathBuf::from(".")
                } else {
                    p.to_owned()
                }
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let scan_dir = if input.ends_with('/') {
            input_path
        } else {
            fallback_parent.clone()
        };

        // Don't try to list the dir if it doesn't exist.
        let entries = if scan_dir.is_dir() {
            match std::fs::read_dir(scan_dir) {
                Ok(read_dir) => Ok(read_dir),
                Err(err) if err.kind() == ErrorKind::NotFound => std::fs::read_dir(fallback_parent),
                Err(err) => Err(err),
            }?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };

        let mut idx = 0;
        let limit = 15;

        while idx < entries.len() && self.paths.len() < limit {
            let entry = entries.get(idx).unwrap();

            let path = entry.path();
            let path_str = if path.is_dir() {
                format!("{}/", path.to_string_lossy())
            } else {
                path.to_string_lossy().to_string()
            };

            if path_str.starts_with(&self.input) && path_str.len() != self.input.len() {
                self.paths.push(path_str);
            }

            idx = idx.saturating_add(1);
        }

        self.lcp = self.longest_common_prefix();

        Ok(())
    }

    fn longest_common_prefix(&self) -> String {
        let mut ret: String = String::new();

        let mut sorted = self.paths.clone();
        sorted.sort();
        if sorted.is_empty() {
            return ret;
        }

        let mut first_word = sorted.first().unwrap().chars();
        let mut last_word = sorted.last().unwrap().chars();

        loop {
            match (first_word.next(), last_word.next()) {
                (Some(c1), Some(c2)) if c1 == c2 => {
                    ret.push(c1);
                }
                _ => return ret,
            }
        }
    }
}

impl Autocomplete for FilepathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;

        Ok(self.paths.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;

        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => match self.lcp.is_empty() {
                true => Replacement::None,
                false => Replacement::Some(self.lcp.clone()),
            },
        })
    }
}

trait ExpandTildeExt {
    fn expand_tilde(&self) -> Option<PathBuf>;
}

impl<P> ExpandTildeExt for P
where
    P: AsRef<Path>,
{
    fn expand_tilde(&self) -> Option<PathBuf> {
        let p = self.as_ref();
        if !p.starts_with("~") {
            return Some(p.to_path_buf());
        }
        if p == Path::new("~") {
            return dirs::home_dir().map(|mut x| {
                x.push("");
                x
            });
        }
        dirs::home_dir().map(|mut h| {
            if h == Path::new("/") {
                // Corner case: `h` root directory;
                // don't prepend extra `/`, just drop the tilde.
                p.strip_prefix("~").unwrap().to_path_buf()
            } else {
                h.push(p.strip_prefix("~/").unwrap());
                h
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_expand_tilde() -> Result<()> {
        let mut fpc = FilepathCompleter {
            input: "".to_owned(),
            paths: vec![],
            lcp: "".to_owned(),
        };
        let res = fpc.update_input("~");
        println!("res: {res:?}");
        println!("fpc: {fpc:?}");

        assert!(fpc.paths.is_empty());
        assert!(fpc.lcp.is_empty());
        Ok(())
    }
}
