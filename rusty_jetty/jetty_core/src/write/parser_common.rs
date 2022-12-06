use std::{cmp::max, collections::VecDeque};

use anyhow::{anyhow, Result};
use yaml_peg::NodeRc;

/// This function makes it easy to present messages with surrounding context in a given byte array
///
/// e.g.
///
///
/// ```text
/// "type" is not a string:
/// 8:16
/// - tab:workbook2
/// - name: tab:project1/project2/ambiguous
///   type:
/// ~~-~~-~^
///       - tableau:workbook
/// remove_from:
/// ```
pub(crate) fn indicated_msg(doc: &[u8], mut pos: u64, lines_of_context: usize) -> String {
    let mut line_buffer = VecDeque::new();
    let mut lines_after_counter = 0;
    let mut column = 0;
    let mut error_line = usize::MAX;
    let mut hit_line = false;

    for (line, str_line) in doc.split(|c| *c == b'\n').enumerate() {
        line_buffer.push_back(str_line);

        if line_buffer.len() > lines_of_context * 2 + 1 {
            line_buffer.pop_front();
        };

        let full_line = str_line.len() as u64 + 1;
        if full_line > pos {
            hit_line = true
        }
        if hit_line {
            lines_after_counter += 1;
            column = max(column, pos);
            error_line = line - lines_after_counter;

            if lines_after_counter > lines_of_context {
                break;
            }
        } else {
            pos -= full_line;
        }
    }

    let lines_after = line_buffer
        .range(line_buffer.len() - lines_after_counter + 1..line_buffer.len())
        .map(|l| String::from_utf8_lossy(l))
        .collect::<Vec<_>>()
        .join("\n");
    let lines_before = line_buffer
        // this feels convoluted, but it makes sure that no more than the appropriate number of lines are shown before an error
        .range(
            max(
                0,
                line_buffer.len() - lines_after_counter - lines_of_context,
            )..line_buffer.len() - lines_after_counter,
        )
        .map(|l| String::from_utf8_lossy(l))
        .collect::<Vec<_>>()
        .join("\n");

    let start_red = "\u{1b}[31m";
    let end_red = "\u{1b}[39m";
    format!(
        "{}:{}\n{}\n{}\n{}{}^{}\n{}\n",
        error_line + 1,
        column + 1,
        lines_before,
        String::from_utf8_lossy(line_buffer[line_buffer.len() - lines_after_counter]),
        start_red,
        "~".repeat(column as usize),
        end_red,
        lines_after
    )
}

pub(crate) fn get_optional_string(
    node: NodeRc,
    field_name: &str,
    config: &String,
) -> Result<Option<String>> {
    Ok(if let Ok(val) = node.get(field_name) {
        Some(
            val.as_str()
                .map_err(|_| {
                    anyhow!(
                        "\"{}\" is not a string: {}",
                        field_name,
                        indicated_msg(config.as_bytes(), val.pos(), 2)
                    )
                })?
                .to_owned(),
        )
    } else {
        None
    })
}

pub(crate) fn get_optional_bool(
    node: NodeRc,
    field_name: &str,
    config: &String,
) -> Result<Option<bool>> {
    Ok(if let Ok(val) = node.get(field_name) {
        Some(
            val.as_bool()
                .map_err(|_| {
                    anyhow!(
                        "\"{}\" is not a boolean: {}",
                        field_name,
                        indicated_msg(config.as_bytes(), val.pos(), 2)
                    )
                })?
                .to_owned(),
        )
    } else {
        None
    })
}
