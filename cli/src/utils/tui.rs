use anyhow::Result;
use colored::*;
use console::Term;

/// Fixed-frame TUI helper for the interactive `devc gen` flow.
///
/// Draws a header always pinned at the top and a footer/menu always pinned
/// at the bottom. The prompt area is cleared and the cursor is positioned
/// roughly in the middle so dialoguer prompts render neatly inside the frame.
pub struct Tui {
    term: Term,
    title: String,
    version: String,
}

impl Tui {
    pub fn new(title: &str) -> Self {
        Self {
            term: Term::stdout(),
            title: title.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Clear the screen and redraw header + footer for a given step.
    pub fn draw_frame(&self, step: &str, template: Option<&str>) -> Result<()> {
        self.term.clear_screen()?;
        self.term.move_cursor_to(0, 0)?;

        let (rows, cols) = self.term.size();
        let width = cols as usize;

        self.draw_header(width, step, template)?;
        self.position_cursor_for_prompt(rows)?;
        self.draw_footer(width, rows)?;

        // Reposition cursor in the prompt area (just below the header).
        self.term.move_cursor_to(0, 6)?;

        Ok(())
    }

    /// Clean up the TUI frame after the interactive flow is done.
    pub fn cleanup(&self) -> Result<()> {
        let (rows, _) = self.term.size();
        // Move cursor below the footer before printing normal output.
        let safe_row = rows.saturating_sub(2);
        self.term.move_cursor_to(0, safe_row as usize)?;
        self.term.show_cursor()?;
        Ok(())
    }

    fn draw_header(&self, width: usize, step: &str, template: Option<&str>) -> Result<()> {
        let top_line = "━".repeat(width);
        let bottom_line = "━".repeat(width);

        let title = format!("{} v{}", self.title, self.version);
        let template_info = template.map(|t| format!("Template: {}", t.cyan())).unwrap_or_else(|| "Seleccionando template…".to_string());
        let step_info = format!("Paso: {}", step.yellow());

        println!("{}", top_line.cyan());
        println!("{}", title.bold().cyan());
        println!("  {}  │  {}", step_info, template_info);
        println!("{}", bottom_line.cyan());
        println!();

        Ok(())
    }

    fn draw_footer(&self, width: usize, rows: u16) -> Result<()> {
        let sep = "─".repeat(width);
        let nav = format!("{} {}   {} {}", "Navegar:".dimmed(), "↑ / ↓".cyan(), "Seleccionar:".dimmed(), "Enter".cyan());
        let back = format!("{} {}", "Volver:".dimmed(), "Esc / q".cyan());
        let help = format!("{} {}", "Ayuda:".dimmed(), "devc gen --help".cyan());

        let line1 = format!("{}   {}   {}", nav, back, help);

        let footer_row = rows.saturating_sub(3) as usize;
        self.term.move_cursor_to(0, footer_row)?;
        println!("{}", sep.dimmed());
        println!("{}", line1);
        println!("{}", " devc — DevContainer Generator ".dimmed().italic());

        Ok(())
    }

    fn position_cursor_for_prompt(&self, rows: u16) -> Result<()> {
        // Reserve 4 header rows + blank line = row 5; prompt starts around row 7.
        // Ensure we don't draw over footer (which occupies last 3 rows).
        let prompt_row = 7usize.min(rows.saturating_sub(5) as usize);
        self.term.move_cursor_to(0, prompt_row)?;
        Ok(())
    }
}

/// Convenience for quick status messages that do not need the full frame.
pub fn print_step_banner(step: &str) {
    let width = 70;
    let sep = "─".repeat(width);
    println!();
    println!("{}", sep.dimmed());
    println!("  {} {}", "Paso:".dimmed(), step.yellow());
    println!("{}", sep.dimmed());
}
