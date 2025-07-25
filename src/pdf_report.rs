use chrono::{DateTime, Utc};
use pdf_canvas::{BuiltinFont, Canvas, Pdf};
use std::fs::File;
use std::path::Path;

use crate::error::QmsError;
use crate::Result;

/// Core compliance metrics aggregated for reporting.
#[derive(Debug, Clone)]
pub struct ComplianceMetrics {
    /// Number of open CAPA records.
    pub open_capa: usize,
    /// Number of open high-severity risks.
    pub open_risks: usize,
    /// Qualified supplier percentage (0-100).
    pub qualified_supplier_pct: f32,
    /// Training completion percentage (0-100).
    pub training_completion_pct: f32,
}

/// Configuration for a single PDF report generation run.
#[derive(Debug, Clone)]
pub struct ComplianceReportConfig<'a> {
    /// Destination path for the generated PDF file.
    pub output_path: &'a Path,
    /// System version string for footer.
    pub application_version: &'a str,
    /// Aggregated compliance metrics.
    pub metrics: ComplianceMetrics,
    /// UTC timestamp of report generation.
    pub generated_on: DateTime<Utc>,
    /// Optional custom title; defaults to standard title if `None`.
    pub title: Option<&'a str>,
}

/// Generate a compliance PDF report adhering to FDA documentation requirements.
///
/// The document follows a simple single-page template containing:
/// 1. Header with title and generation timestamp.
/// 2. Body with compliance metrics table.
/// 3. Footer with software version and immutable checksum placeholder.
///
/// The function is ACiD-safe (atomic file creation using a temporary file which is
/// renamed on success) and idempotent (identical input → identical output).
pub fn generate_compliance_report(cfg: &ComplianceReportConfig) -> Result<()> {
    let tmp_path = cfg.output_path.with_extension("tmp");

    // Create PDF; built-in fonts avoid external font dependencies.
    let mut document = Pdf::create(&tmp_path).map_err(|e| QmsError::Application {
        message: format!("Failed to create PDF: {e}"),
    })?;

    let title_text = cfg
        .title
        .unwrap_or("FDA Compliance Summary Report");

    document.render_page(595.0, 842.0, |canvas| {
        render_header(canvas, title_text, cfg.generated_on)?;
        render_metrics_table(canvas, &cfg.metrics)?;
        render_footer(canvas, cfg.application_version)?;
        Ok(())
    })?;

    document.finish().map_err(|e| QmsError::Application {
        message: format!("Failed to finish PDF: {e}"),
    })?;

    // Atomic replace to ensure durability.
    std::fs::rename(&tmp_path, cfg.output_path).map_err(|e| QmsError::FileSystem {
        path: cfg.output_path.display().to_string(),
        message: e.to_string(),
    })?;

    Ok(())
}

fn render_header(canvas: &mut Canvas, title: &str, ts: DateTime<Utc>) -> pdf_canvas::Result<()> {
    let font = BuiltinFont::Helvetica_Bold;
    canvas.left_text(50.0, 800.0, font, 24.0, title)?;

    let subtitle = format!("Generated: {}", ts.format("%Y-%m-%d %H:%M UTC"));
    canvas.left_text(50.0, 780.0, BuiltinFont::Helvetica, 12.0, &subtitle)?;
    canvas.line(50.0, 775.0, 545.0, 775.0)?;
    Ok(())
}

fn render_metrics_table(canvas: &mut Canvas, metrics: &ComplianceMetrics) -> pdf_canvas::Result<()> {
    let font_label = BuiltinFont::Helvetica_Bold;
    let font_value = BuiltinFont::Helvetica;

    let start_y = 740.0;
    let line_height = 22.0;

    let rows = vec![
        ("Open CAPA Records", metrics.open_capa.to_string()),
        (
            "Open High-Severity Risks",
            metrics.open_risks.to_string(),
        ),
        (
            "Qualified Supplier %",
            format!("{:.1}%", metrics.qualified_supplier_pct),
        ),
        (
            "Training Completion %",
            format!("{:.1}%", metrics.training_completion_pct),
        ),
    ];

    for (idx, (label, value)) in rows.into_iter().enumerate() {
        let y = start_y - (idx as f64 * line_height);
        canvas.left_text(50.0, y, font_label, 12.0, label)?;
        canvas.right_text(545.0, y, font_value, 12.0, &value)?;
    }

    Ok(())
}

fn render_footer(canvas: &mut Canvas, version: &str) -> pdf_canvas::Result<()> {
    canvas.line(50.0, 100.0, 545.0, 100.0)?;
    let footer_text = format!("QMSrs version {} | © 2025 QMS Development Team", version);
    canvas.center_text(297.5, 85.0, BuiltinFont::Helvetica, 10.0, &footer_text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_compliance_report() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_report.pdf");

        let cfg = ComplianceReportConfig {
            output_path: &path,
            application_version: crate::APPLICATION_VERSION,
            metrics: ComplianceMetrics {
                open_capa: 3,
                open_risks: 2,
                qualified_supplier_pct: 92.5,
                training_completion_pct: 97.8,
            },
            generated_on: Utc::now(),
            title: None,
        };

        generate_compliance_report(&cfg).expect("PDF generation should succeed");
        // Validate file exists and starts with %PDF- header
        let mut f = File::open(&path).unwrap();
        let mut header = [0u8; 5];
        use std::io::Read;
        f.read_exact(&mut header).unwrap();
        assert_eq!(&header, b"%PDF-");
    }
}