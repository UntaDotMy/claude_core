//! Purpose: Operator and advanced help text rendering for the native CLI.
//! Caller: commands.rs `run_help_command` and the bare-invocation path in `Application::run`.
//! Dependencies: std::io::{self, Write}; embedded help_*.txt resources.
//! Main Functions: render_help_surface.
//! Side Effects: Writes embedded help text to the supplied writer..

use std::io::{self, Write};

static OPERATOR_HELP_COMMAND_LINES: &str = include_str!("help_operator.txt");
static ADVANCED_HELP_COMMAND_LINES: &str = include_str!("help_advanced.txt");
static OPERATOR_MIGRATION_STATE_LINES: &str = include_str!("help_operator_state.txt");
static ADVANCED_MIGRATION_STATE_LINES: &str = include_str!("help_advanced_state.txt");

pub fn render_help_surface<W: Write + ?Sized>(
    output_writer: &mut W,
    include_advanced: bool,
) -> io::Result<()> {
    writeln!(output_writer, "claude-skills")?;
    writeln!(output_writer)?;
    if include_advanced {
        writeln!(output_writer, "Help mode: advanced")?;
    } else {
        writeln!(output_writer, "Help mode: operator")?;
    }
    writeln!(output_writer)?;
    writeln!(output_writer, "Operator commands:")?;
    write_help_lines(output_writer, OPERATOR_HELP_COMMAND_LINES)?;
    writeln!(output_writer)?;
    writeln!(output_writer, "Advanced surfaces:")?;
    writeln!(output_writer, "  help advanced")?;
    writeln!(
        output_writer,
        "  Use this when you need orchestration, memory, or memoriesv2 internals instead of the default operator path."
    )?;
    if include_advanced {
        writeln!(output_writer)?;
        writeln!(output_writer, "Advanced commands:")?;
        write_help_lines(output_writer, ADVANCED_HELP_COMMAND_LINES)?;
    }
    writeln!(output_writer)?;
    writeln!(output_writer, "Current migration state:")?;
    write_help_lines(output_writer, OPERATOR_MIGRATION_STATE_LINES)?;
    if include_advanced {
        write_help_lines(output_writer, ADVANCED_MIGRATION_STATE_LINES)?;
    }
    Ok(())
}

fn write_help_lines<W: Write + ?Sized>(output_writer: &mut W, lines_body: &str) -> io::Result<()> {
    for line in lines_body.split_inclusive('\n') {
        output_writer.write_all(line.as_bytes())?;
    }
    Ok(())
}
