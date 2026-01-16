# PLAN-4: Update Notification

**Phase**: 4 of 5
**Milestone**: M1 - Auto-Update Foundation
**Created**: 2026-01-16

## Overview

Add update notification UI to the menu system:
- Pass update check result to Menu
- Show update status in header/footer
- Add dynamic "Update CLI" menu option when update available
- Highlight update option with color

## Prerequisites

- [x] Phase 3 completed (UpdateChecker, background task)
- [x] Menu uses ratatui with header/menu/footer layout
- [x] UpdateCheckResult available via Arc<RwLock<Option<UpdateCheckResult>>>

## Current State

| Component | Status |
|-----------|--------|
| menu.rs | Has ratatui TUI with header/footer |
| main.rs | Has update_result Arc, spawns background check |
| updater.rs | Has UpdateCheckResult with update_available, latest_version |

## Tasks

<task type="auto" priority="p0">
  <n>Add update status to Menu struct</n>
  <files>
    cli/src/menu.rs,
    cli/src/main.rs
  </files>
  <action>
    1. Update Menu struct to store optional update info:
       ```rust
       pub struct Menu {
           state: ListState,
           items: Vec<(String, MenuChoice)>,  // Change to String for dynamic labels
           sb_path: Option<PathBuf>,
           sdisk_path: Option<PathBuf>,
           update_available: Option<String>,  // Latest version if update available
       }
       ```

    2. Add MenuChoice::UpdateCLI variant:
       ```rust
       #[derive(Debug, Clone, PartialEq)]
       pub enum MenuChoice {
           RunSB,
           RunSDisk,
           UpdateBinaries,
           UpdateCLI,  // New - self-update
           Settings,
           Plugins,
           Exit,
       }
       ```

    3. Add method to set update status:
       ```rust
       pub fn set_update_status(&mut self, latest_version: Option<String>) {
           self.update_available = latest_version;
           self.rebuild_items();
       }

       fn rebuild_items(&mut self) {
           self.items = vec![
               ("Run Saorsa Browser (sb)".to_string(), MenuChoice::RunSB),
               ("Run Saorsa Disk (sdisk)".to_string(), MenuChoice::RunSDisk),
               ("Update Binaries".to_string(), MenuChoice::UpdateBinaries),
           ];

           // Add update option if available
           if let Some(version) = &self.update_available {
               self.items.push((
                   format!("Update CLI to v{}", version),
                   MenuChoice::UpdateCLI,
               ));
           }

           self.items.extend([
               ("Settings".to_string(), MenuChoice::Settings),
               ("Plugins".to_string(), MenuChoice::Plugins),
               ("Exit".to_string(), MenuChoice::Exit),
           ]);
       }
       ```

    4. Update main.rs to pass update result to Menu:
       ```rust
       // Before menu loop
       if let Some(result) = update_result.read().await.as_ref() {
           if result.update_available {
               menu.set_update_status(result.latest_version.clone());
           }
       }
       ```

    5. Handle MenuChoice::UpdateCLI in main.rs (placeholder for now):
       ```rust
       MenuChoice::UpdateCLI => {
           println!("Self-update will be implemented in Phase 5");
           println!("Press Enter to continue...");
           let mut input = String::new();
           std::io::stdin().read_line(&mut input)?;
       }
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
  </verify>
  <done>
    - Menu has update_available field
    - MenuChoice::UpdateCLI variant added
    - Dynamic menu items based on update status
    - Main loop handles UpdateCLI choice
    - Zero clippy warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add visual update indicator to menu</n>
  <files>
    cli/src/menu.rs
  </files>
  <action>
    1. Update draw_header() to show update notification:
       ```rust
       fn draw_header(&self, f: &mut Frame, area: Rect) {
           let mut lines = vec![
               Line::from(vec![Span::styled(
                   "Saorsa CLI",
                   Style::default()
                       .fg(Color::Cyan)
                       .add_modifier(Modifier::BOLD),
               )]),
               Line::from(vec![Span::raw("Interactive menu for Saorsa tools")]),
           ];

           // Add update notification if available
           if let Some(version) = &self.update_available {
               lines.push(Line::from(vec![
                   Span::styled(
                       format!("Update available: v{}", version),
                       Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                   ),
               ]));
           }

           let header = Paragraph::new(lines)
               .alignment(Alignment::Center)
               .block(Block::default().borders(Borders::BOTTOM));

           f.render_widget(header, area);
       }
       ```

    2. Update draw_menu() to highlight UpdateCLI option:
       ```rust
       // In the map closure for items:
       MenuChoice::UpdateCLI => {
           style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
       }
       ```

    3. Adjust header constraint to accommodate update line:
       ```rust
       // In draw():
       let header_height = if self.update_available.is_some() { 4 } else { 3 };
       let chunks = Layout::default()
           .direction(Direction::Vertical)
           .margin(2)
           .constraints([
               Constraint::Length(header_height),
               Constraint::Min(10),
               Constraint::Length(4),
           ])
           .split(f.area());
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo build -p cli
    cargo run -p cli -- --help
  </verify>
  <done>
    - Header shows update notification when available
    - UpdateCLI menu item highlighted in yellow
    - Layout adjusts for update notification
    - No clippy warnings
  </done>
</task>

<task type="auto" priority="p2">
  <n>Refresh update status in menu loop</n>
  <files>
    cli/src/main.rs
  </files>
  <action>
    1. Check for update result at start of each menu iteration:
       ```rust
       loop {
           // Refresh update status (background task may have completed)
           if let Some(result) = update_result.read().await.as_ref() {
               if result.update_available {
                   menu.set_update_status(result.latest_version.clone());
               }
           }

           // Check for binaries and update menu
           let config_read = config.read().await;
           let (sb_path, sdisk_path) = check_binaries(...).await?;
           drop(config_read);
           menu.set_binary_paths(sb_path.clone(), sdisk_path.clone());

           // Show menu and get choice
           let choice = menu.run().await?;
           ...
       }
       ```

    2. Remove the initial delay and notification print (now handled by menu):
       ```rust
       // Remove this block:
       // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
       // if let Some(result) = update_result.read().await.as_ref() {...}
       ```
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p cli --all-features -- -D warnings
    cargo test -p cli
    cargo run -p cli -- --version
  </verify>
  <done>
    - Update status refreshed each menu iteration
    - No initial delay/print needed
    - All tests pass
    - Zero clippy warnings
  </done>
</task>

## Exit Criteria

- [x] MenuChoice::UpdateCLI variant exists
- [x] Menu shows update notification in header when available
- [x] UpdateCLI menu item appears dynamically
- [x] UpdateCLI item highlighted in yellow
- [x] Main loop handles UpdateCLI (placeholder message)
- [x] All tests pass (19 tests)
- [x] Zero clippy warnings

## Notes

- Phase 5 will implement actual self-update when UpdateCLI is selected
- Using yellow color for update notifications (matches common conventions)
- Dynamic menu items require switching from &'static str to String

## Next Phase

Phase 5: Self-Update & Restart
- Implement binary replacement logic
- Add restart prompt after self-update
- Handle platform-specific restart (Unix exec, Windows spawn)
