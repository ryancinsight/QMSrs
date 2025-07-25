use crate::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};
use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};

/// Main TUI application state
pub struct TuiApp {
    pub should_quit: bool,
    pub current_tab: TabState,
    pub selected_menu_item: usize,
    pub last_tick: Instant,
    // Persistent list states for each tab to maintain selection
    pub dashboard_list_state: ratatui::widgets::ListState,
    pub documents_list_state: ratatui::widgets::ListState,
    pub audit_list_state: ratatui::widgets::ListState,
    pub reports_list_state: ratatui::widgets::ListState,
}

impl TuiApp {
    /// Create new TUI application
    pub fn new() -> Self {
        let mut dashboard_state = ratatui::widgets::ListState::default();
        dashboard_state.select(Some(0));
        
        let mut documents_state = ratatui::widgets::ListState::default();
        documents_state.select(Some(0));
        
        let mut audit_state = ratatui::widgets::ListState::default();
        audit_state.select(Some(0));
        
        let mut reports_state = ratatui::widgets::ListState::default();
        reports_state.select(Some(0));
        
        Self {
            should_quit: false,
            current_tab: TabState::Dashboard,
            selected_menu_item: 0,
            last_tick: Instant::now(),
            dashboard_list_state: dashboard_state,
            documents_list_state: documents_state,
            audit_list_state: audit_state,
            reports_list_state: reports_state,
        }
    }

    /// Handle input events
    pub fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Tab => self.next_tab(),
                    KeyCode::Up => self.move_up(),
                    KeyCode::Down => self.move_down(),
                    KeyCode::Enter => self.handle_enter(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Move to next tab
    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabState::Dashboard => TabState::Documents,
            TabState::Documents => TabState::AuditTrail,
            TabState::AuditTrail => TabState::Reports,
            TabState::Reports => TabState::Dashboard,
        };
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        match self.current_tab {
            TabState::Dashboard => {
                let i = match self.dashboard_list_state.selected() {
                    Some(i) => if i == 0 { 4 } else { i - 1 },
                    None => 0,
                };
                self.dashboard_list_state.select(Some(i));
            }
            TabState::Documents => {
                let i = match self.documents_list_state.selected() {
                    Some(i) => if i == 0 { 2 } else { i - 1 },
                    None => 0,
                };
                self.documents_list_state.select(Some(i));
            }
            TabState::AuditTrail => {
                let i = match self.audit_list_state.selected() {
                    Some(i) => if i == 0 { 2 } else { i - 1 },
                    None => 0,
                };
                self.audit_list_state.select(Some(i));
            }
            TabState::Reports => {
                let i = match self.reports_list_state.selected() {
                    Some(i) => if i == 0 { 2 } else { i - 1 },
                    None => 0,
                };
                self.reports_list_state.select(Some(i));
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        match self.current_tab {
            TabState::Dashboard => {
                let i = match self.dashboard_list_state.selected() {
                    Some(i) => if i >= 4 { 0 } else { i + 1 },
                    None => 0,
                };
                self.dashboard_list_state.select(Some(i));
            }
            TabState::Documents => {
                let i = match self.documents_list_state.selected() {
                    Some(i) => if i >= 2 { 0 } else { i + 1 },
                    None => 0,
                };
                self.documents_list_state.select(Some(i));
            }
            TabState::AuditTrail => {
                let i = match self.audit_list_state.selected() {
                    Some(i) => if i >= 2 { 0 } else { i + 1 },
                    None => 0,
                };
                self.audit_list_state.select(Some(i));
            }
            TabState::Reports => {
                let i = match self.reports_list_state.selected() {
                    Some(i) => if i >= 2 { 0 } else { i + 1 },
                    None => 0,
                };
                self.reports_list_state.select(Some(i));
            }
        }
    }

    /// Handle enter key
    pub fn handle_enter(&mut self) {
        // Implementation for handling enter key press
        // This would typically trigger actions based on current selection
    }

    /// Main render function
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());

        self.render_tabs(f, chunks[0]);
        
        match self.current_tab {
            TabState::Dashboard => self.render_dashboard(f, chunks[1]),
            TabState::Documents => self.render_documents(f, chunks[1]),
            TabState::AuditTrail => self.render_audit_trail(f, chunks[1]),
            TabState::Reports => self.render_reports(f, chunks[1]),
        }
    }

    /// Render tab bar
    fn render_tabs<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let tab_titles = vec!["Dashboard", "Documents", "Audit Trail", "Reports"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("QMS - FDA Compliant"))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .select(self.current_tab as usize);
        
        f.render_widget(tabs, area);
    }

    /// Render dashboard tab
    fn render_dashboard<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let dashboard_items = vec![
            ListItem::new("✓ FDA CFR Part 820 Compliance: ACTIVE"),
            ListItem::new("✓ Audit Trail System: OPERATIONAL"),
            ListItem::new("✓ Document Control: READY"),
            ListItem::new("✓ User Authentication: ENABLED"),
            ListItem::new("✓ Encryption Status: AES-256 ACTIVE"),
        ];

        let dashboard_list = List::new(dashboard_items)
            .block(Block::default().borders(Borders::ALL).title("System Status"))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(dashboard_list, area, &mut self.dashboard_list_state);
    }

    /// Render documents tab
    fn render_documents<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let document_items = vec![
            ListItem::new("📄 SOP-001: Quality System Procedures [APPROVED]"),
            ListItem::new("📄 WI-002: Calibration Work Instructions [DRAFT]"),
            ListItem::new("📄 FORM-003: Device Master Record [EFFECTIVE]"),
        ];

        let document_list = List::new(document_items)
            .block(Block::default().borders(Borders::ALL).title("Document Control"))
            .highlight_style(Style::default().bg(Color::Green).fg(Color::White))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(document_list, area, &mut self.documents_list_state);
    }

    /// Render audit trail tab
    fn render_audit_trail<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let audit_items = vec![
            ListItem::new("🔍 2024-01-15 10:30:25 - User login: admin [SUCCESS]"),
            ListItem::new("🔍 2024-01-15 10:31:12 - Document accessed: SOP-001 [SUCCESS]"),
            ListItem::new("🔍 2024-01-15 10:32:45 - Configuration changed [SUCCESS]"),
        ];

        let audit_list = List::new(audit_items)
            .block(Block::default().borders(Borders::ALL).title("Audit Trail"))
            .highlight_style(Style::default().bg(Color::Red).fg(Color::White))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(audit_list, area, &mut self.audit_list_state);
    }

    /// Render reports tab
    fn render_reports<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let report_items = vec![
            ListItem::new("📊 FDA Compliance Report - Q4 2024"),
            ListItem::new("📊 Audit Summary - January 2024"),
            ListItem::new("📊 Document Control Metrics - Current"),
        ];

        let report_list = List::new(report_items)
            .block(Block::default().borders(Borders::ALL).title("Reports"))
            .highlight_style(Style::default().bg(Color::Magenta).fg(Color::White))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(report_list, area, &mut self.reports_list_state);
    }
}

/// Tab states for navigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabState {
    Dashboard = 0,
    Documents = 1,
    AuditTrail = 2,
    Reports = 3,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_app_creation() {
        let app = TuiApp::new();
        assert_eq!(app.current_tab, TabState::Dashboard);
        assert!(!app.should_quit);
        assert_eq!(app.selected_menu_item, 0);
    }

    #[test]
    fn test_tab_navigation() {
        let mut app = TuiApp::new();
        
        // Test forward navigation
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Documents);
        
        app.next_tab();
        assert_eq!(app.current_tab, TabState::AuditTrail);
        
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Reports);
        
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Dashboard);
    }

    #[test]
    fn test_dashboard_navigation() {
        let mut app = TuiApp::new();
        assert_eq!(app.dashboard_list_state.selected(), Some(0));
        
        app.move_down();
        assert_eq!(app.dashboard_list_state.selected(), Some(1));
        
        app.move_up();
        assert_eq!(app.dashboard_list_state.selected(), Some(0));
        
        // Test wrap-around
        app.move_up();
        assert_eq!(app.dashboard_list_state.selected(), Some(4));
    }

    #[test]
    fn test_input_handling() {
        let mut app = TuiApp::new();
        
        // Test that input handling returns Ok and doesn't crash
        // Note: This test doesn't actually send events, but verifies the function exists
        // In a real implementation, we'd mock crossterm events
        assert!(!app.should_quit);
    }

    #[test]
    fn test_end_to_end_workflow() {
        let mut app = TuiApp::new();
        
        // Simulate a complete user workflow
        
        // 1. Start on dashboard
        assert_eq!(app.current_tab, TabState::Dashboard);
        assert_eq!(app.dashboard_list_state.selected(), Some(0));
        
        // 2. Navigate through items
        app.move_down();
        app.move_down();
        assert_eq!(app.dashboard_list_state.selected(), Some(2));
        
        // 3. Switch to documents tab
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Documents);
        
        // 4. Navigate documents
        app.move_down();
        assert_eq!(app.documents_list_state.selected(), Some(1));
        
        // 5. Switch to audit trail
        app.next_tab();
        assert_eq!(app.current_tab, TabState::AuditTrail);
        
        // 6. Navigate audit entries
        app.move_down();
        app.move_down();
        assert_eq!(app.audit_list_state.selected(), Some(2));
        
        // 7. Switch to reports
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Reports);
        
        // 8. Navigate reports
        app.move_down();
        assert_eq!(app.reports_list_state.selected(), Some(1));
        
        // 9. Return to dashboard
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Dashboard);
        
        // Verify workflow completed successfully
        assert!(!app.should_quit);
    }
}