use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::Stylize;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use regex::Regex;
use std::{
    io::{self, Write},
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Clone)]
struct BootEntry {
    id: String,
    name: String,
}

#[derive(Clone, Copy)]
enum Focus {
    Priority,
    BootOnce,
}

#[derive(Clone)]
enum Action {
    None,
    SetOrder(Vec<String>),
    BootOnce(String),
}

enum UIState {
    Main,
    AskPassword,
    PasswordError,
    ConfirmReboot,
    CountdownReboot(u8),
    QuitConfirm,
    Help,
    ErrorMessage(String),
}

fn execute_sudo_command(args: &[&str], password: &str) -> Result<(bool, String), Box<dyn std::error::Error>> {
    let mut child = Command::new("sudo")
        .arg("-S")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(password.as_bytes())?;
        stdin.write_all(b"\n")?;
        stdin.flush()?;
        drop(stdin);
    }
    
    let output = child.wait_with_output()?;
    
    let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
    
    if stderr_text.contains("Sorry") || stderr_text.contains("try again") {
        return Ok((false, "Incorrect password".to_string()));
    }
    
    if !output.status.success() {
        let error_msg = if !stderr_text.trim().is_empty() {
            stderr_text.trim().to_string()
        } else {
            format!("Command failed with exit code: {}", output.status.code().unwrap_or(-1))
        };
        return Ok((false, error_msg));
    }
    
    Ok((true, String::new()))
}

fn execute_set_boot_order(order_ids: &[String], password: &str) -> Result<UIState, Box<dyn std::error::Error>> {
    let order = order_ids.join(",");
    let result = execute_sudo_command(&["efibootmgr", "-o", &order], password)?;
    
    if result.0 {
        Ok(UIState::ConfirmReboot)
    } else if result.1 == "Incorrect password" {
        Ok(UIState::PasswordError)
    } else {
        Ok(UIState::ErrorMessage(result.1))
    }
}

fn execute_boot_once(id: &str, password: &str) -> Result<UIState, Box<dyn std::error::Error>> {
    let result = execute_sudo_command(&["efibootmgr", "-n", id], password)?;
    
    if result.0 {
        Ok(UIState::CountdownReboot(5))
    } else if result.1 == "Incorrect password" {
        Ok(UIState::PasswordError)
    } else {
        Ok(UIState::ErrorMessage(result.1))
    }
}

fn center(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width / 2 - width / 2,
        area.y + area.height / 2 - height / 2,
        width,
        height,
    )
}

fn centered_area(area: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let w = area.width * width_pct / 100;
    let h = area.height * height_pct / 100;
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

fn fetch_boot_entries() -> Result<Vec<BootEntry>, Box<dyn std::error::Error>> {
    let output = Command::new("efibootmgr")
        .arg("-v")
        .output()?;

    if !output.status.success() {
        return Err("Failed to run efibootmgr. Are you running on a UEFI system?".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let regex = Regex::new(r"Boot(?P<id>[0-9A-Fa-f]{4})\*?\s+(?P<name>[^\t\(]+)").unwrap();

    let entries = text.lines()
        .filter_map(|line| {
            regex.captures(line).map(|cap| BootEntry {
                id: cap["id"].trim().to_string(),
                name: cap["name"].trim().to_string(),
            })
        })
        .collect();
    
    Ok(entries)
}

fn fetch_boot_order() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("efibootmgr")
        .output()?;
    
    if !output.status.success() {
        return Err("Failed to run efibootmgr".into());
    }
    
    let text = String::from_utf8_lossy(&output.stdout);

    let order = text.lines()
        .find(|l| l.starts_with("BootOrder:"))
        .map(|l| {
            l["BootOrder:".len()..]
                .trim()
                .split(',')
                .map(|s| s.trim().to_string())
                .collect()
        })
        .unwrap_or_default();
    
    Ok(order)
}

fn draw_main_ui(
    f: &mut ratatui::Frame,
    area: Rect,
    entries: &[BootEntry],
    focus: Focus,
    selected_priority: usize,
    selected_boot_once: usize,
    current_boot_id: &str,
) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ])
        .split(area);

    // Title
    f.render_widget(
        Paragraph::new("Boot Switcher")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center),
        layout[0],
    );

    // Priority panel
    let priority_items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let style = if matches!(focus, Focus::Priority) && i == selected_priority {
                Style::default().bg(Color::Cyan).fg(Color::Black).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let marker = if e.id == current_boot_id { " →" } else { "  " };
            ListItem::new(format!("{} {}. {}", marker, i + 1, e.name)).style(style)
        })
        .collect();

    f.render_widget(
        List::new(priority_items).block(
            Block::default()
                .title(" Boot Priority (default order) ")
                .borders(Borders::ALL),
        ),
        layout[1],
    );

    // Boot once panel
    let boot_once_items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let style = if matches!(focus, Focus::BootOnce) && i == selected_boot_once {
                Style::default().bg(Color::Cyan).fg(Color::Black).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let marker = if e.id == current_boot_id { " →" } else { "  "};
            ListItem::new(format!("{} {}", marker, e.name)).style(style)
        })
        .collect();

    f.render_widget(
        List::new(boot_once_items)
            .block(Block::default().title(" Boot Once ").borders(Borders::ALL)),
        layout[2],
    );

    let footer = "Tab: Switch panel  |  ↑↓/jk: Move  |  u/d: Reorder  |  Enter: Apply/Boot  |  ?: Help  |  q: Quit";
    f.render_widget(
        Paragraph::new(footer)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        layout[3],
    );
}

fn draw_password_popup(f: &mut ratatui::Frame, area: Rect, password: &str, show: bool) {
    let popup_width = area.width * 3 / 4;
    let popup_height = 6;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Authentication "),
        popup,
    );

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width - 2,
            height: popup.height - 2,
        });

    f.render_widget(
        Paragraph::new("Enter sudo password")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner[0],
    );

    let displayed = if show {
        password.to_string()
    } else {
        "*".repeat(password.len())
    };

    let bar_width = popup_width / 2;
    let bar_area = Rect {
        x: popup.x + (popup.width - bar_width) / 2,
        y: inner[2].y,
        width: bar_width,
        height: 1,
    };

    f.render_widget(
        Paragraph::new(format!(" {}", displayed))
            .style(
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black),
            )
            .alignment(Alignment::Left),
        bar_area,
    );

    let help_area = Rect {
        x: area.x,
        y: popup.y + popup_height + 1,
        width: area.width,
        height: 1,
    };

    f.render_widget(
        Paragraph::new("Enter = Confirm  |  Esc = Cancel  |  Tab = Show/Hide")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        help_area,
    );
}

fn draw_reboot_popup(f: &mut ratatui::Frame, area: Rect, yes_selected: bool) {
    let popup_width = area.width / 3;
    let popup_height = 7;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Apply Complete "),
        popup,
    );

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width - 2,
            height: popup.height - 2,
        });

    f.render_widget(
        Paragraph::new("Reboot now?")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner[0],
    );

    let buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner[1]);

    let yes_style = if yes_selected {
        Style::default().bg(Color::Green).fg(Color::Black).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let no_style = if !yes_selected {
        Style::default().bg(Color::Red).fg(Color::Black).bold()
    } else {
        Style::default().fg(Color::White)
    };

    f.render_widget(
        Paragraph::new("[ Yes ]")
            .alignment(Alignment::Center)
            .style(yes_style),
        buttons[0],
    );
    f.render_widget(
        Paragraph::new("[ No ]")
            .alignment(Alignment::Center)
            .style(no_style),
        buttons[1],
    );
}

fn draw_processing_screen(f: &mut ratatui::Frame, area: Rect) {
    let popup_width = area.width / 3;
    let popup_height = 5;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Paragraph::new("Processing...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan).bold())
            .block(Block::default().borders(Borders::ALL)),
        popup,
    );
}

fn draw_password_error_popup(f: &mut ratatui::Frame, area: Rect) {
    let popup_width = area.width / 2;
    let popup_height = 7;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Authentication Failed ")
            .style(Style::default().fg(Color::Red)),
        popup,
    );

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width - 2,
            height: popup.height - 2,
        });

    f.render_widget(
        Paragraph::new("Incorrect password!")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red).bold()),
        inner[0],
    );

    f.render_widget(
        Paragraph::new("Please try again.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner[1],
    );

    f.render_widget(
        Paragraph::new("Press any key to continue")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        inner[2],
    );
}

fn draw_countdown_screen(f: &mut ratatui::Frame, area: Rect, seconds: u8) {
    let popup_width = area.width / 2;
    let popup_height = 8;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Rebooting ")
            .style(Style::default().fg(Color::Cyan)),
        popup,
    );

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width - 2,
            height: popup.height - 2,
        });

    f.render_widget(
        Paragraph::new(format!("Rebooting in {} second{}...", seconds, if seconds == 1 { "" } else { "s" }))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner[0],
    );

    let progress = (5 - seconds) as f32 / 5.0;
    let bar_width = (popup_width - 10) as f32 * progress;
    let filled = "█".repeat(bar_width as usize);
    let empty = "░".repeat((popup_width - 10) as usize - bar_width as usize);
    
    f.render_widget(
        Paragraph::new(format!("{}{}", filled, empty))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan)),
        inner[1],
    );

    f.render_widget(
        Paragraph::new("Press Esc to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        inner[2],
    );
}

fn draw_quit_confirm_popup(f: &mut ratatui::Frame, area: Rect, yes_selected: bool) {
    let popup_width = area.width / 3;
    let popup_height = 7;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Quit ")
            .style(Style::default().fg(Color::Yellow)),
        popup,
    );

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(2)])
        .split(Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width - 2,
            height: popup.height - 2,
        });

    f.render_widget(
        Paragraph::new("Quit without applying?")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner[0],
    );

    let buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner[1]);

    let yes_style = if yes_selected {
        Style::default().bg(Color::Red).fg(Color::Black).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let no_style = if !yes_selected {
        Style::default().bg(Color::Green).fg(Color::Black).bold()
    } else {
        Style::default().fg(Color::White)
    };

    f.render_widget(
        Paragraph::new("[ Yes ]")
            .alignment(Alignment::Center)
            .style(yes_style),
        buttons[0],
    );
    f.render_widget(
        Paragraph::new("[ No ]")
            .alignment(Alignment::Center)
            .style(no_style),
        buttons[1],
    );
}

fn draw_help_screen(f: &mut ratatui::Frame, area: Rect) {
    let popup_width = area.width * 3 / 4;
    let popup_height = 23;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .style(Style::default().fg(Color::Cyan)),
        popup,
    );

    let help_text = vec![
        "",
        "Navigation:",
        "  Tab              Switch between panels",
        "  ↑/↓ or k/j       Move selection up/down",
        "",
        "Boot Priority Panel:",
        "  u/d              Move entry up/down in boot order",
        "  Enter            Apply new boot order (requires reboot)",
        "",
        "Boot Once Panel:",
        "  Enter            Boot selected entry on next reboot only",
        "",
        "Password Dialog:",
        "  Tab              Toggle password visibility",
        "  Enter            Confirm",
        "  Esc              Cancel",
        "",
        "General:",
        "  ? or h           Show this help screen",
        "  q                Quit application",
        "",
        "Press any key to close this help screen",
    ];

    let inner = Rect {
        x: popup.x + 2,
        y: popup.y + 1,
        width: popup.width - 4,
        height: popup.height - 2,
    };

    f.render_widget(
        Paragraph::new(help_text.join("\n"))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left),
        inner,
    );
}

fn draw_error_message_popup(f: &mut ratatui::Frame, area: Rect, error_msg: &str) {
    let popup_width = area.width * 2 / 3;
    let popup_height = 9;
    let popup = center(area, popup_width, popup_height);

    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .title(" Error ")
            .style(Style::default().fg(Color::Red)),
        popup,
    );

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(Rect {
            x: popup.x + 1,
            y: popup.y + 1,
            width: popup.width - 2,
            height: popup.height - 2,
        });

    f.render_widget(
        Paragraph::new("Command failed:")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red).bold()),
        inner[0],
    );

    f.render_widget(
        Paragraph::new(error_msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White)),
        inner[1],
    );

    f.render_widget(
        Paragraph::new("Press any key to continue")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        inner[2],
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = fetch_boot_entries()?;
    let order = fetch_boot_order()?;

    let current_boot_id = order.first().cloned().unwrap_or_default();

    if !order.is_empty() {
        entries.sort_by_key(|e| {
            order
                .iter()
                .position(|id| id == &e.id)
                .unwrap_or(usize::MAX)
        });
    }

    let mut selected_priority = 0usize;
    let mut selected_boot_once = 0usize;
    let mut focus = Focus::Priority;

    let mut state = UIState::Main;
    let mut password = String::new();
    let mut show_password = false;
    let mut pending_action = Action::None;
    let mut reboot_yes = true;
    let mut quit_yes = false;
    let original_order: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
    let mut last_tick = std::time::Instant::now();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let area = centered_area(f.area(), 65, 60);

            match &state {
                UIState::Main => draw_main_ui(
                    f,
                    area,
                    &entries,
                    focus,
                    selected_priority,
                    selected_boot_once,
                    &current_boot_id,
                ),
                UIState::AskPassword => draw_password_popup(f, area, &password, show_password),
                UIState::PasswordError => draw_password_error_popup(f, area),
                UIState::ConfirmReboot => draw_reboot_popup(f, area, reboot_yes),
                UIState::CountdownReboot(seconds) => draw_countdown_screen(f, area, *seconds),
                UIState::QuitConfirm => draw_quit_confirm_popup(f, area, quit_yes),
                UIState::Help => draw_help_screen(f, area),
                UIState::ErrorMessage(msg) => draw_error_message_popup(f, area, msg),
            }
        })?;

        if let UIState::CountdownReboot(seconds) = state {
            if last_tick.elapsed() >= Duration::from_secs(1) {
                last_tick = std::time::Instant::now();
                if seconds > 1 {
                    state = UIState::CountdownReboot(seconds - 1);
                } else {
                    let mut reboot = Command::new("sudo")
                        .arg("reboot")
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()?;
                    let _ = reboot.wait();
                    break;
                }
            }
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match state {
                    UIState::Main => match key.code {
                        KeyCode::Char('q') => {
                            let current_order: Vec<String> = entries.iter().map(|e| e.id.clone()).collect();
                            let has_changes = current_order != original_order;
                            if has_changes {
                                state = UIState::QuitConfirm;
                                quit_yes = false;
                            } else {
                                break;
                            }
                        }

                        KeyCode::Tab => {
                            focus = match focus {
                                Focus::Priority => Focus::BootOnce,
                                Focus::BootOnce => Focus::Priority,
                            }
                        }

                        KeyCode::Up | KeyCode::Char('k') => match focus {
                            Focus::Priority if selected_priority > 0 => selected_priority -= 1,
                            Focus::BootOnce if selected_boot_once > 0 => selected_boot_once -= 1,
                            _ => {}
                        },

                        KeyCode::Down | KeyCode::Char('j') => match focus {
                            Focus::Priority if selected_priority + 1 < entries.len() => {
                                selected_priority += 1
                            }
                            Focus::BootOnce if selected_boot_once + 1 < entries.len() => {
                                selected_boot_once += 1
                            }
                            _ => {}
                        },

                        KeyCode::Char('u') if matches!(focus, Focus::Priority) => {
                            if selected_priority > 0 {
                                entries.swap(selected_priority, selected_priority - 1);
                                selected_priority -= 1;
                            }
                        }

                        KeyCode::Char('d') if matches!(focus, Focus::Priority) => {
                            if selected_priority + 1 < entries.len() {
                                entries.swap(selected_priority, selected_priority + 1);
                                selected_priority += 1;
                            }
                        }

                        KeyCode::Enter => {
                            pending_action = match focus {
                                Focus::Priority => {
                                    let ids =
                                        entries.iter().map(|e| e.id.clone()).collect::<Vec<_>>();
                                    Action::SetOrder(ids)
                                }
                                Focus::BootOnce => {
                                    let id = entries[selected_boot_once].id.clone();
                                    Action::BootOnce(id)
                                }
                            };
                            password.clear();
                            state = UIState::AskPassword;
                        }

                        KeyCode::Char('?') | KeyCode::Char('h') => {
                            state = UIState::Help;
                        }

                        _ => {}
                    },

                    UIState::AskPassword => match key.code {
                        KeyCode::Esc => {
                            password.clear();
                            pending_action = Action::None;
                            state = UIState::Main;
                        }
                        KeyCode::Tab => {
                            show_password = !show_password;
                        }
                        KeyCode::Backspace => {
                            password.pop();
                        }
                        KeyCode::Enter => {
                            terminal.draw(|f| {
                                let area = centered_area(f.area(), 65, 60);
                                draw_processing_screen(f, area);
                            })?;
                            
                            state = match pending_action.clone() {
                                Action::SetOrder(order_ids) => {
                                    execute_set_boot_order(&order_ids, &password)?
                                }
                                Action::BootOnce(id) => {
                                    execute_boot_once(&id, &password)?
                                }
                                Action::None => UIState::Main,
                            };
                            
                            if matches!(state, UIState::PasswordError | UIState::ErrorMessage(_)) {
                                password.clear();
                            }
                        },
                        KeyCode::Char(c) => password.push(c),
                        _ => {}
                    },

                    UIState::PasswordError => {
                        state = UIState::AskPassword;
                    }

                    UIState::ConfirmReboot => match key.code {
                        KeyCode::Esc => {
                            state = UIState::Main;
                        }
                        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                            reboot_yes = !reboot_yes;
                        }
                        KeyCode::Enter => {
                            if reboot_yes {
                                state = UIState::CountdownReboot(5);
                                last_tick = std::time::Instant::now();
                            } else {
                                state = UIState::Main;
                            }
                        }
                        _ => {}
                    },

                    UIState::CountdownReboot(_) => {
                        if let KeyCode::Esc = key.code {
                            state = UIState::Main;
                        }
                    }

                    UIState::QuitConfirm => match key.code {
                        KeyCode::Esc => {
                            state = UIState::Main;
                        }
                        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                            quit_yes = !quit_yes;
                        }
                        KeyCode::Enter => {
                            if quit_yes {
                                break;
                            } else {
                                state = UIState::Main;
                            }
                        }
                        _ => {}
                    }

                    UIState::Help => {
                        state = UIState::Main;
                    }

                    UIState::ErrorMessage(_) => {
                        state = UIState::AskPassword;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
