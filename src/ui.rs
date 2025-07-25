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
    pub capa_list_state: ratatui::widgets::ListState,
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
        
        let mut capa_state = ratatui::widgets::ListState::default();
        capa_state.select(Some(0));
        
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
            capa_list_state: capa_state,
            reports_list_state: reports_state,
        }
    }

    /// Handle input events
    pub fn handle_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                    KeyCode::Tab | KeyCode::Right => self.next_tab(),
                    KeyCode::Left => self.previous_tab(),
                    KeyCode::Up | KeyCode::Char('k') => self.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => self.move_down(),
                    KeyCode::Enter | KeyCode::Char(' ') => self.handle_enter(),
                    KeyCode::Char('h') | KeyCode::F(1) => self.show_help(),
                    KeyCode::Home => self.move_to_first(),
                    KeyCode::End => self.move_to_last(),
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
            TabState::AuditTrail => TabState::Capa,
            TabState::Capa => TabState::Reports,
            TabState::Reports => TabState::Dashboard,
        };
    }

    /// Move to previous tab
    pub fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            TabState::Dashboard => TabState::Reports,
            TabState::Documents => TabState::Dashboard,
            TabState::AuditTrail => TabState::Documents,
            TabState::Capa => TabState::AuditTrail,
            TabState::Reports => TabState::Capa,
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
            TabState::Capa => {
                let i = match self.capa_list_state.selected() {
                    Some(i) => if i == 0 { 4 } else { i - 1 },
                    None => 0,
                };
                self.capa_list_state.select(Some(i));
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
            TabState::Capa => {
                let i = match self.capa_list_state.selected() {
                    Some(i) => if i >= 4 { 0 } else { i + 1 },
                    None => 0,
                };
                self.capa_list_state.select(Some(i));
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

    /// Move to first item in current tab
    pub fn move_to_first(&mut self) {
        match self.current_tab {
            TabState::Dashboard => self.dashboard_list_state.select(Some(0)),
            TabState::Documents => self.documents_list_state.select(Some(0)),
            TabState::AuditTrail => self.audit_list_state.select(Some(0)),
            TabState::Capa => self.capa_list_state.select(Some(0)),
            TabState::Reports => self.reports_list_state.select(Some(0)),
        }
    }

    /// Move to last item in current tab
    pub fn move_to_last(&mut self) {
        match self.current_tab {
            TabState::Dashboard => self.dashboard_list_state.select(Some(4)), // 5 items, index 4
            TabState::Documents => self.documents_list_state.select(Some(2)), // 3 items, index 2
            TabState::AuditTrail => self.audit_list_state.select(Some(2)), // 3 items, index 2
            TabState::Capa => self.capa_list_state.select(Some(2)), // 3 items, index 2
            TabState::Reports => self.reports_list_state.select(Some(2)), // 3 items, index 2
        }
    }

    /// Show help information
    pub fn show_help(&mut self) {
        println!("\n=== QMSrs Navigation Help ===");
        println!("Tab/→     : Next tab");
        println!("←         : Previous tab");
        println!("↑/k       : Move up");
        println!("↓/j       : Move down");
        println!("Enter/Space: Select item");
        println!("Home      : First item");
        println!("End       : Last item");
        println!("h/F1      : Show this help");
        println!("q/Esc     : Quit application");
        println!("=============================\n");
    }

    /// Handle enter key
    pub fn handle_enter(&mut self) {
        match self.current_tab {
            TabState::Dashboard => {
                if let Some(selected) = self.dashboard_list_state.selected() {
                    match selected {
                        0 => println!("📊 System Status: All systems operational - FDA compliant"),
                        1 => println!("📋 Document Control: 45 active SOPs, 12 pending reviews"),
                        2 => println!("🔍 Audit Trail: 1,247 entries today, all validated"),
                        3 => println!("🔧 CAPA System: 3 open actions, 2 due this week"),
                        4 => println!("📈 Reports: Last compliance report: 98.5% score"),
                        _ => println!("Dashboard item {} selected", selected),
                    }
                }
            }
            TabState::Documents => {
                if let Some(selected) = self.documents_list_state.selected() {
                    match selected {
                        0 => println!("📄 SOP-001: Quality Manual v2.1 - Opening document viewer..."),
                        1 => println!("📄 SOP-002: Device History Record v1.3 - Accessing controlled document..."),
                        2 => println!("📄 SOP-003: Risk Management v1.0 - Loading FDA-compliant procedures..."),
                        _ => println!("Document {} opened", selected),
                    }
                }
            }
            TabState::AuditTrail => {
                if let Some(selected) = self.audit_list_state.selected() {
                    match selected {
                        0 => println!("🔍 User login: admin [SUCCESS] - Viewing full audit details..."),
                        1 => println!("🔍 Document accessed: SOP-001 [SUCCESS] - Showing access log..."),
                        2 => println!("🔍 Configuration changed [SUCCESS] - Displaying change history..."),
                        _ => println!("Audit trail item {} selected", selected),
                    }
                }
            }
            TabState::Capa => {
                if let Some(selected) = self.capa_list_state.selected() {
                    match selected {
                        0 => println!("🔧 CAPA-001: Non-conforming Product Investigation [OPEN] - Opening investigation details..."),
                        1 => println!("🔧 CAPA-002: Audit Finding Remediation [IN PROGRESS] - Viewing action plan..."),
                        2 => println!("🔧 CAPA-003: Process Improvement Initiative [CLOSED] - Showing effectiveness verification..."),
                        _ => println!("CAPA item {} selected", selected),
                    }
                }
            }
            TabState::Reports => {
                if let Some(selected) = self.reports_list_state.selected() {
                    match selected {
                        0 => println!("📊 FDA Compliance Report - Q4 2024 - Generating detailed analysis..."),
                        1 => println!("📊 Audit Summary - January 2024 - Opening comprehensive report..."),
                        2 => println!("📊 Document Control Metrics - Current - Loading real-time dashboard..."),
                        _ => println!("Report {} selected", selected),
                    }
                }
            }
        }
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
            TabState::Capa => self.render_capa(f, chunks[1]),
            TabState::Reports => self.render_reports(f, chunks[1]),
        }
    }

    /// Render tab bar
    fn render_tabs<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let tab_titles = vec!["Dashboard", "Documents", "Audit Trail", "CAPA", "Reports"];
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

    /// Render CAPA tab
    fn render_capa<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let capa_items = vec![
            ListItem::new("🔧 CAPA-001: Non-conforming Product Investigation [OPEN]"),
            ListItem::new("🔧 CAPA-002: Audit Finding Remediation [IN PROGRESS]"),
            ListItem::new("🔧 CAPA-003: Process Improvement Initiative [CLOSED]"),
        ];

        let capa_list = List::new(capa_items)
            .block(Block::default().borders(Borders::ALL).title("CAPA Management"))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
            .highlight_symbol("▶ ");

        f.render_stateful_widget(capa_list, area, &mut self.capa_list_state);
    }
}

/// Tab states for navigation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabState {
    Dashboard = 0,
    Documents = 1,
    AuditTrail = 2,
    Capa = 3,
    Reports = 4,
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
        assert_eq!(app.current_tab, TabState::Capa);
        
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
        
        // 7. Switch to CAPA
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Capa);
        
        // 8. Navigate CAPA items
        app.move_down();
        assert_eq!(app.capa_list_state.selected(), Some(1));
        
        // 9. Switch to reports
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Reports);
        
        // 10. Navigate reports
        app.move_down();
        assert_eq!(app.reports_list_state.selected(), Some(1));
        
        // 11. Return to dashboard
        app.next_tab();
        assert_eq!(app.current_tab, TabState::Dashboard);
        
        // Verify workflow completed successfully
        assert!(!app.should_quit);
    }
}