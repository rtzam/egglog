use std::io::{self, Read};
use std::io::{BufRead, BufReader, Write};

use crate::{all_sexps, Context, EGraph};

impl EGraph {
    pub fn repl(&mut self) -> io::Result<()> {
        self.repl_with(io::stdin(), io::stdout())
    }

    pub fn repl_with<R, W>(&mut self, input: R, mut output: W) -> io::Result<()>
    where
        R: Read,
        W: Write,
    {
        let mut cmd_buffer = String::new();

        for line in BufReader::new(input).lines() {
            let line_str = line?;
            cmd_buffer.push_str(&line_str);
            cmd_buffer.push('\n');
            // handles multi-line commands
            if should_eval(&cmd_buffer) {
                run_command_in_scripting(self, &cmd_buffer, &mut output)?;
                cmd_buffer = String::new();
            }
        }

        if !cmd_buffer.is_empty() {
            run_command_in_scripting(self, &cmd_buffer, &mut output)?;
        }

        Ok(())
    }
}

fn should_eval(curr_cmd: &str) -> bool {
    all_sexps(Context::new(None, curr_cmd)).is_ok()
}

fn run_command_in_scripting<W>(egraph: &mut EGraph, command: &str, mut output: W) -> io::Result<()>
where
    W: Write,
{
    match egraph.parse_and_run_program(None, command) {
        Ok(msgs) => {
            for msg in msgs {
                writeln!(output, "{msg}")?;
            }
        }
        Err(err) => log::error!("{err}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_eval() {
        #[rustfmt::skip]
        let test_cases = vec![
            vec![
                "(extract",
                "\"1",
                ")",
                "(",
                ")))",
                "\"",
                ";; )",
                ")"
            ],
            vec![
                "(extract 1) (extract",
                "2) (",
                "extract 3) (extract 4) ;;;; ("
            ],
            vec![
                "(extract \"\\\")\")"
            ]];
        for test in test_cases {
            let mut cmd_buffer = String::new();
            for (i, line) in test.iter().enumerate() {
                cmd_buffer.push_str(line);
                cmd_buffer.push('\n');
                assert_eq!(should_eval(&cmd_buffer), i == test.len() - 1);
            }
        }
    }

    #[test]
    fn test_repl() {
        let mut egraph = EGraph::default();

        let input = "(extract 1)";
        let mut output = Vec::new();
        egraph.repl_with(input.as_bytes(), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "1\n");

        let input = "\n\n\n";
        let mut output = Vec::new();
        egraph.repl_with(input.as_bytes(), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "");

        let input = "(set-option interactive_mode 1)";
        let mut output = Vec::new();
        egraph.repl_with(input.as_bytes(), &mut output).unwrap();
        assert_eq!(String::from_utf8(output).unwrap(), "(done)\n");

        let input = "(set-option interactive_mode 1)\n(extract 1)(extract 2)\n";
        let mut output = Vec::new();
        egraph.repl_with(input.as_bytes(), &mut output).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "(done)\n1\n(done)\n2\n(done)\n"
        );
    }
}
