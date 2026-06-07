use anyhow::Result;
use colored::*;
use console::{Key, Term};

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

    /// Print contextual help lines right below the header (for Input / Password prompts).
    pub fn print_context(&self, lines: &[String]) -> Result<()> {
        println!("  {}", "┌─ Contexto ─".cyan());
        for line in lines {
            println!("  {} {}", "│".cyan(), line.italic().cyan());
        }
        println!("  {}", "└─────────────".cyan());
        println!();
        Ok(())
    }

    /// Show a centered help popup and wait for any key.
    pub fn show_help_box(&self, step: &str) -> Result<()> {
        let (rows, cols) = self.term.size();
        let term_width = cols as usize;
        let term_height = rows as usize;

        let content = get_step_context(step).unwrap_or_else(|| {
            vec![
                "No hay información adicional para este paso.".to_string(),
            ]
        });

        let box_width = content.iter().map(|l| l.len()).max().unwrap_or(40).clamp(40, term_width.saturating_sub(8));
        let box_height = content.len() + 4; // title + separator + content + padding

        let start_row = (term_height.saturating_sub(box_height)) / 2;
        let start_col = (term_width.saturating_sub(box_width)) / 2;

        let top = format!("┌{}┐", "─".repeat(box_width));
        let bottom = format!("└{}┘", "─".repeat(box_width));
        let title = format!("{:<width$}", " ℹ️  Ayuda ", width = box_width);
        let separator = format!("├{}┤", "─".repeat(box_width));

        self.term.hide_cursor()?;

        // Draw box
        self.term.move_cursor_to(start_col, start_row)?;
        println!("{}", top.cyan());
        self.term.move_cursor_to(start_col, start_row + 1)?;
        println!("│{}│", title.bold().cyan());
        self.term.move_cursor_to(start_col, start_row + 2)?;
        println!("{}", separator.cyan());

        for (i, line) in content.iter().enumerate() {
            let padded = format!(" {:<width$}", line, width = box_width.saturating_sub(1));
            self.term.move_cursor_to(start_col, start_row + 3 + i)?;
            println!("│{}│", padded);
        }

        let hint = format!(" {:<width$}", "Presioná Enter para volver…", width = box_width.saturating_sub(1));
        self.term.move_cursor_to(start_col, start_row + 3 + content.len())?;
        println!("│{}│", hint.dimmed());

        self.term.move_cursor_to(start_col, start_row + 4 + content.len())?;
        println!("{}", bottom.cyan());

        // Wait for Enter
        loop {
            if let Ok(key) = self.term.read_key() {
                if matches!(key, Key::Enter | Key::Escape | Key::Char('q') | Key::Char(' ')) {
                    break;
                }
            }
        }

        self.term.show_cursor()?;
        Ok(())
    }

    /// Clean up the TUI frame after the interactive flow is done.
    pub fn cleanup(&self) -> Result<()> {
        let (rows, _) = self.term.size();
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
        let help = format!("{} {}", "Ayuda:".dimmed(), "❓ última opción".cyan());

        let line1 = format!("{}   {}   {}", nav, back, help);

        let footer_row = rows.saturating_sub(3) as usize;
        self.term.move_cursor_to(0, footer_row)?;
        println!("{}", sep.dimmed());
        println!("{}", line1);
        println!("{}", " devc — DevContainer Generator ".dimmed().italic());

        Ok(())
    }

    fn position_cursor_for_prompt(&self, rows: u16) -> Result<()> {
        let prompt_row = 7usize.min(rows.saturating_sub(5) as usize);
        self.term.move_cursor_to(0, prompt_row)?;
        Ok(())
    }
}

/// Return contextual help lines for a given step name.
pub fn get_step_context(step: &str) -> Option<Vec<String>> {
    let lines: Vec<String> = match step {
        "Selección de template" => vec![
            "Los templates definen el entorno de desarrollo base.".to_string(),
            "Incluyen el sistema operativo, herramientas y configuraciones predefinidas.".to_string(),
            "Elegí uno acorde al stack tecnológico de tu proyecto.".to_string(),
        ],
        "Nombre del proyecto" => vec![
            "Este nombre se usará para carpetas y configuraciones internas.".to_string(),
            "Por defecto se sugiere el nombre del directorio actual.".to_string(),
        ],
        "Ubicación del devcontainer" => vec![
            "Indicá dónde querés que se generen los archivos del devcontainer.".to_string(),
            "Se creará la carpeta si no existe.".to_string(),
        ],
        "Configuración de Git" => vec![
            "Estos datos se usarán para configurar git dentro del contenedor.".to_string(),
            "Son opcionales; si los omitís, podés configurarlos después.".to_string(),
        ],
        "Configuración de seguridad" => vec![
            "Definí cómo se gestionan los usuarios y permisos dentro del contenedor.".to_string(),
            "Developer: usuario común con sudo sin contraseña.".to_string(),
            "Secure: usuario común sin sudo.".to_string(),
            "Root: todo se ejecuta como root.".to_string(),
        ],
        "Configuración de red" => vec![
            "Definí cómo se conecta el contenedor a la red.".to_string(),
            "Bridge: red aislada con port mapping (más seguro, recomendado).".to_string(),
            "Host: comparte la red del host directamente (máximo rendimiento, sin aislamiento).".to_string(),
            "None: contenedor sin acceso a la red (máximo aislamiento).".to_string(),
        ],
        s if s.contains("Usuario de desarrollo") || s == "Usuario de desarrollo" => vec![
            "REMOTE_USER es el usuario con el que VS Code se conecta al contenedor.".to_string(),
            "Este usuario tendrá permisos para trabajar con el código fuente.".to_string(),
            "Por defecto: 'developer', 'node' (Node.js) o 'root' (Android).".to_string(),
        ],
        s if s.contains("Contraseñas del contenedor") || s == "Contraseñas del contenedor" => vec![
            "Podés definir contraseñas personalizadas o dejar que se generen automáticamente.".to_string(),
            "Se necesitan dos contraseñas: una para el usuario de desarrollo y otra para root.".to_string(),
            "Las contraseñas auto-generadas tienen 12 caracteres alfanuméricos.".to_string(),
        ],
        s if s.contains("Contraseña de root") || s == "Contraseña de root" => vec![
            "Esta contraseña permite acceder como root dentro del contenedor.".to_string(),
            "Es útil para instalar paquetes del sistema o modificar configuraciones.".to_string(),
            "Presioná Enter sin escribir para auto-generar una segura.".to_string(),
        ],
        s if s.contains("Privilegios sudo") || s == "Privilegios sudo" => vec![
            "NOPASSWD: el usuario puede usar sudo sin ingresar contraseña.".to_string(),
            "Password: se requiere contraseña para cada comando sudo.".to_string(),
            "None: el usuario no tiene acceso a sudo.".to_string(),
        ],
        s if s.contains("Guardar credenciales") || s == "Guardar credenciales" => vec![
            "Las credenciales se guardan en ~/.devc/credentials/<proyecto>.json".to_string(),
            "El archivo tiene permisos 600 (solo lectura para el dueño).".to_string(),
            "Útil para recordar contraseñas generadas automáticamente.".to_string(),
        ],
        s if s.contains("Personalizar versiones") || s == "Personalizar versiones" => vec![
            "Podés cambiar las versiones de herramientas definidas en el Dockerfile.".to_string(),
            "Por ejemplo: versión de Node, Java, Android SDK, etc.".to_string(),
            "Las versiones seleccionadas se inyectan durante la construcción.".to_string(),
        ],
        _ => return None,
    };
    Some(lines)
}
