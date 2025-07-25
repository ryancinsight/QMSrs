use crate::{Result, QmsError};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};
use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};

/// Main TUI application state
pub struct TuiApp {
    pub should_quit: bool,
    pub current_tab: TabState,
    pub selected_menu_item: usize,
    pub last_tick: Instant,
}

impl TuiApp {
    /// Create new TUI application
    pub fn new() -> Self {
        Self {
            should_quit: false,
            current_tab: TabState::Dashboard,
            selected_menu_item: 0,
            last_tick: Instant::now(),
        }
    }

    /// Handle input events
    pub fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Tab => self.next_tab(),
                    KeyCode::BackTab => self.previous_tab(),
                    KeyCode::Up => self.previous_menu_item(),
                    KeyCode::Down => self.next_menu_item(),
                    KeyCode::Enter => self.handle_menu_selection()?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Move to next tab
    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabState::Dashboard => TabState::Documents,
            TabState::Documents => TabState::AuditTrail,
            TabState::AuditTrail => TabState::Reports,
            TabState::Reports => TabState::Settings,
            TabState::Settings => TabState::Dashboard,
        };
        self.selected_menu_item = 0;
    }

    /// Move to previous tab
    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabState::Dashboard => TabState::Settings,
            TabState::Documents => TabState::Dashboard,
            TabState::AuditTrail => TabState::Documents,
            TabState::Reports => TabState::AuditTrail,
            TabState::Settings => TabState::Reports,
        };
        self.selected_menu_item = 0;
    }

    /// Move to previous menu item
    fn previous_menu_item(&mut self) {
        if self.selected_menu_item > 0 {
            self.selected_menu_item -= 1;
        }
    }

    /// Move to next menu item
    fn next_menu_item(&mut self) {
        let max_items = self.menu_item_count();
        if self.selected_menu_item + 1 < max_items {
            self.selected_menu_item += 1;
        }
    }

    /// Handle menu item selection
    fn handle_menu_selection(&mut self) -> Result<()> {
        match self.current_tab {
            TabState::Dashboard => {
                // Handle dashboard actions
            }
            TabState::Documents => {
                // Handle document actions
            }
            TabState::AuditTrail => {
                // Handle audit trail actions
            }
            TabState::Reports => {
                // Handle report actions
            }
            TabState::Settings => {
                // Handle settings actions
            }
        }
        Ok(())
    }

    /// Render the TUI
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        // Render tabs
        self.render_tabs(f, chunks[0]);

        // Render current tab content
        match self.current_tab {
            TabState::Dashboard => self.render_dashboard(f, chunks[1]),
            TabState::Documents => self.render_documents(f, chunks[1]),
            TabState::AuditTrail => self.render_audit_trail(f, chunks[1]),
            TabState::Reports => self.render_reports(f, chunks[1]),
            TabState::Settings => self.render_settings(f, chunks[1]),
        }
    }

    /// Render tab bar
    fn render_tabs<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let titles = vec!["Dashboard", "Documents", "Audit Trail", "Reports", "Settings"]
            .iter()
            .cloned()
            .map(Line::from)
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("QMS - FDA Compliant"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(self.current_tab as usize);

        f.render_widget(tabs, area);
    }

    /// Render dashboard tab
    fn render_dashboard<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);

        // System status
        let status_items = vec![
            ListItem::new("System Status: Operational"),
            ListItem::new("FDA Compliance: Active"),
            ListItem::new("Audit Trail: Enabled"),
            ListItem::new("Last Backup: 2 hours ago"),
            ListItem::new("Active Users: 3"),
        ];

        let status_list = List::new(status_items)
            .block(Block::default().title("System Status").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_widget(status_list, chunks[0]);

        // Recent activities
        let activity_items = vec![
            ListItem::new("Document SOP-001 approved by Quality Manager"),
            ListItem::new("User john.doe logged in"),
            ListItem::new("Audit report generated for Q1 2024"),
            ListItem::new("Document FORM-001 created"),
        ];

        let activity_list = List::new(activity_items)
            .block(Block::default().title("Recent Activities").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(activity_list, chunks[1]);
    }

    /// Render documents tab
    fn render_documents<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let document_items = vec![
            ListItem::new("SOP-001: Quality Management System Overview (v1.0) - Effective"),
            ListItem::new("SOP-002: Document Control Procedure (v2.1) - Effective"),
            ListItem::new("FORM-001: Change Request Form (v1.0) - Draft"),
            ListItem::new("WI-001: Equipment Calibration Work Instruction (v1.5) - Effective"),
        ];

        let document_list = List::new(document_items)
            .block(Block::default().title("Controlled Documents").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_stateful_widget(document_list, area, &mut self.get_list_state());
    }

    /// Render audit trail tab
    fn render_audit_trail<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let audit_items = vec![
            ListItem::new("2024-01-15 10:30:25 | user123 | LOGIN | auth_system | SUCCESS"),
            ListItem::new("2024-01-15 10:31:12 | user123 | CREATE_DOCUMENT | SOP-003 | SUCCESS"),
            ListItem::new("2024-01-15 10:35:45 | manager456 | APPROVE_DOCUMENT | SOP-001 | SUCCESS"),
            ListItem::new("2024-01-15 10:40:18 | auditor789 | VIEW_AUDIT_TRAIL | system | SUCCESS"),
        ];

        let audit_list = List::new(audit_items)
            .block(Block::default().title("Audit Trail").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_stateful_widget(audit_list, area, &mut self.get_list_state());
    }

    /// Render reports tab
    fn render_reports<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let report_items = vec![
            ListItem::new("FDA Compliance Report - Q1 2024"),
            ListItem::new("Document Status Summary - January 2024"),
            ListItem::new("Audit Trail Integrity Report - Last 30 days"),
            ListItem::new("User Activity Report - Weekly"),
        ];

        let report_list = List::new(report_items)
            .block(Block::default().title("Available Reports").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        f.render_stateful_widget(report_list, area, &mut self.get_list_state());
    }

    /// Render settings tab
    fn render_settings<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let settings_text = vec![
            Line::from(vec![
                Span::styled("FDA Compliance Mode: ", Style::default().fg(Color::Yellow)),
                Span::styled("ENABLED", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Audit Retention: ", Style::default().fg(Color::Yellow)),
                Span::styled("7 years (2555 days)", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Encryption: ", Style::default().fg(Color::Yellow)),
                Span::styled("AES-256-GCM", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Database: ", Style::default().fg(Color::Yellow)),
                Span::styled("SQLite WAL Mode", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from("Press 'q' to quit, Tab to navigate, Enter to select"),
        ];

        let settings_paragraph = Paragraph::new(settings_text)
            .block(Block::default().title("System Settings").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(settings_paragraph, area);
    }

    /// Get list state for highlighting
    fn get_list_state(&self) -> ratatui::widgets::ListState {
        let mut state = ratatui::widgets::ListState::default();
        state.select(Some(self.selected_menu_item));
        state
    }
}

/// Tab state enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabState {
    Dashboard = 0,
    Documents = 1,
    AuditTrail = 2,
    Reports = 3,
    Settings = 4,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_app_creation() {
        let app = TuiApp::new();
        assert!(!app.should_quit);
        assert_eq!(app.current_tab, TabState::Dashboard);
        assert_eq!(app.selected_menu_item, 0);
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = TuiApp::new();
        
        assert_eq!(app.current_tab, TabState::Dashboard);
        
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Documents);
        
        app.previous_tab();
        assert_eq!(app.current_tab, TabState::Dashboard);
    }

    #[test]
    fn test_menu_navigation() {
        let mut app = TuiApp::new();
        
        assert_eq!(app.selected_menu_item, 0);
        
        app.next_menu_item();
        assert_eq!(app.selected_menu_item, 1);
        
        app.previous_menu_item();
        assert_eq!(app.selected_menu_item, 0);
    }
}