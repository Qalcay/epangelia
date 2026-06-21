mod tune;
mod name;
mod stat;
mod fight;
mod enemy;
mod gems;
mod game;
mod exist;
mod id;
mod time;
mod hash;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, poll},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use rand::{Rng, RngExt, SeedableRng, rngs::StdRng};
use std::{io, time::{Duration, Instant}};

const EVAL_GAME_TIME: f64 = 0.11111111111; // days in game per second

struct Board {
    game: game::Game,
    rng: StdRng,
    input: String,
    log: Vec<Line<'static>>,
    does_quit: bool,
    does_idle: bool,
    at_enemy: Option<enemy::Enemy>, // what enemy is being fought
    hp_enemy_remaining: f64, // keep state of enemy hp as it is being fought
    prev_tick: Instant,
    battles: u64, // in game eliminations
}

impl Board {
    fn new() -> Self {
        let mut board = Board {
            game: game::Game::default(),
            rng: StdRng::from_rng(&mut rand::rng()),
            input: String::new(),
            log: Vec::new(),
            does_quit: false,
            does_idle: true,
            at_enemy: None,
            hp_enemy_remaining: 0.0, // why 0?
            prev_tick: Instant::now(),
            battles: 0,
        };
        board.push(" a stranger enters... ");
        board.next_spawn(tune::Rank::Normal);
        board
    }

    fn push(&mut self, s: impl Into<String>) {
        self.log.push(Line::from(s.into()));
        if self.log.len() > 500 { self.log.remove(0); }
    }
    fn push_stylize(&mut self, line: Line<'static>) {
        self.log.push(line);
        if self.log.len() > 500 { self.log.remove(0); }
    }

    fn next_spawn(&mut self, rank: tune::Rank) {
        let e = enemy::Enemy::spawn(self.game.stage, rank, &mut self.rng);
        self.hp_enemy_remaining = e.hp_max;
        self.at_enemy = Some(e);
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.prev_tick).as_secs_f64();
        self.prev_tick = now;
        if !self.does_idle { return; }
        self.game.clock.advance(dt * EVAL_GAME_TIME);

        let enemy = match &self.at_enemy { Some(e) => e.clone(), None => { self.next_spawn(tune::Rank::Normal); return; } };
        let dps = fight::effective_dps(&self.game.equipped, &enemy);
        self.hp_enemy_remaining -= dps * dt;
        if self.hp_enemy_remaining <= 0.0 {
            self.battles += 1;
            let (coin, amt) = enemy.drop(&mut self.rng);
            self.game.add_coin(coin, amt);
            let rank = match self.battles % 21 {
                0 => tune::Rank::Overlord,
                n if n % 11 == 0 => tune::Rank::Boss,
                n if n % 4 == 0 => tune::Rank::Elite,
                _ => tune::Rank::Normal,
            };
            if !matches!(rank, tune::Rank::Normal) {
                self.push_stylize(Line::from(vec![
                    Span::styled(format!("[ {} ]", enemy.name),
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]));
            }
            self.next_spawn(rank);
        }
    }

    fn check_command(&mut self) {
        let raw = self.input.trim().to_string();
        self.input.clear();
        if raw.is_empty() { return; }
        self.push(format!("> {raw}"));
        let parts: Vec<&str> = raw.split_whitespace().collect();
        match parts.as_slice() {
            ["qq!"] => self.does_quit = true,

            ["help"] => self.comd_advice(),
            ["see"] => self.comd_see(),

            ["pause"] => { self.does_idle = !self.does_idle;
                //self.push(if self.does_idle { "" } else { "" });
                }

            ["next"] => { self.game.stage += 1;
                self.game.top_stage = self.game.stage.max(self.game.stage);
                self.next_spawn(tune::Rank::Normal);
                self.push(format!("stage {}", self.game.stage)); }
            ["prev"] => {
                if self.game.stage > 1 {
                    self.game.stage -= 1;
                    self.next_spawn(tune::Rank::Normal);
                    self.push(format!("stage {}", self.game.stage));
                }
            }

            ["gems"] => self.comd_gems(),
            ["buy", g] => { if let Ok(gi) = g.parse::<u8>() { self.comd_store(gi); }
                else { self.push("buy <gem 0-31>"); }},
            ["crack", g, e] => self.comd_crack(g, e),
            ["bag"] => self.comd_bag(),
            ["runs"] => self.comd_runs(),
            ["runs", n] => { let k = n.parse::<usize>().unwrap_or(20); self.comd_runs_n(k); }
            ["load", id ] => self.comd_load(id),
            ["id"] => self.comd_id(),
            ["import", path] => self.comd_import(path),

            ["equip", i] => if let Ok(ix) = i.parse::<usize>() {
                match self.game.equip(ix) { Ok(_) => self.push("equipped"),
                    Err(e) => self.push(e) } } else { self.push("equip <idx>"); },

            ["smelt", i] => if let Ok(ix) = i.parse::<usize>() {
                match self.game.smelt(ix) { Ok(v) => self.push(format!("earned {:.0} copper", v)),
                    Err(e) => self.push(e) } } else { self.push("smelt <idx>"); },

            ["enemy"] => self.comd_enemy(),
            ["happybirthday"] => self.comd_hb(),

            //[""] => self.comd(),
            //[""] => self.comd(),

            _ => return,
            //_ => self.push(""),
        }
        let _ = exist::save(&self.game);
    }

    fn comd_advice(&mut self) {
        for l in [
            "asd"
        ] { self.push(l); }
    }

    fn comd_see(&mut self) {
        let enemy = self.at_enemy.clone();
        self.push(format!("{}", self.game.clock.date_string()));
        self.push(format!("Stage [ {} ] ..| DPS [ {:.0} ] ..|",
            self.game.stage, self.game.raw_loadout_dps()));
        self.push(format!(" "));
        if let Some(e) = &enemy {
            let pct = (self.hp_enemy_remaining / e.hp_max * 100.0).clamp(0.0, 100.0);
            self.push_stylize(Line::from(vec![
                Span::styled(format!(">< {}", e.name), Style::default().fg(Color::Red)),
                Span::raw(format!(" ({:.0}%) [+{}] <-- [{:.0}] dmg/s",
                    pct,
                    gems::element_name(e.resist_element),
                    fight::effective_dps(&self.game.equipped, e))),
            ]));
            for (name, contrib, mult) in fight::breakdown(&self.game.equipped, e) {
                let tag = if mult < 0.5 { "..." } else if mult > 1.2 { "!!!" } else { "o" };
                self.push(format!(" {name}: {contrib:.0} dps (x{mult:.2} {tag})"));
            }
        }
        self.push(format!(" "));
        let equipped = self.game.equipped.clone();
        for (i, w) in equipped.iter().enumerate() {
            self.push_stylize(Line::from(vec![
                Span::styled(format!(" [{}] {}", i, w.name),
                    Style::default().fg(step_color(w.step)).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" [Lv. {}] + [{}]", w.step, gems::element_name(w.mods.element))),
            ]));
        }
    }

    fn comd_enemy(&mut self) {
        if let Some(e) = self.at_enemy.clone() {
            let pct = (self.hp_enemy_remaining / e.hp_max * 100.0).clamp(0.0, 100.0);
            self.push(format!(" Fighting {} -- HP {:.0}%", e.name, pct));
        }
    }

    fn comd_gems(&mut self) {
        self.push(format!("Coins: [ {:.0}c ] ..| Top Stage [ {} ] ..|",
            self.game.net_copper(), self.game.top_stage));
        for tier in 0..4u32 {
            let coin = tune::gem_tier_coin(tier);
            let unlocked = self.game.top_stage >= tune::coin_unlock_stage(coin);
            self.push(format!(" Tier {} Gems ({:?}, stage {}+): {}",
                tier, coin, tune::coin_unlock_stage(coin),
                if unlocked { "Unlocked" } else { "Locked" }));
        }
    }

    fn comd_store(&mut self, gem: u8) {
        match self.game.buy_gem(gem) {
            Ok(_) => self.push(format!(" Bought {} (Gem {})", gems::gem_name(gem), gem)),
            Err(e) => self.push(format!("{e} Not Enough Coins! ")),
        }
    }

    fn comd_crack(&mut self, g: &str, el: &str) {
        let (gem, element) = match (g.parse::<u8>(), el.parse::<u8>()) {
            (Ok(a), Ok(b)) => (a, b),
            _ => { self.push("'crack <gem 0-31> <element 0-31>"); return; }
        };
        let r: f64 = self.rng.random();
        let ceiling = self.game.top_stage.min(tune::MAX_STEP);
        let base_step = (1.0 + (ceiling as f64 - 1.0) * r.powi(3)).round().max(1.0) as u32;
        let crack_mod = self.game.clock.lunation_buff(gem as u64); //self.game.clock.crack_modifier(gem);
        match self.game.roll(gem, element, base_step, crack_mod, &mut self.rng) {
            Ok((w, preserved)) => {
                self.push_stylize(Line::from(vec![
                    Span::raw("-> "),
                    Span::styled(w.name.clone(),
                        Style::default().fg(step_color(w.step)).add_modifier(Modifier::BOLD)),
                    Span::raw(format!(" [step {}]", w.step)),
                ]));
                if preserved { self.push(" the Gem was Unbroken! -- it was Added to Inventory..."); }
            }
            Err(e) => self.push(format!(" Failed to Crack the Gem... {e}")),
        }
    }

    fn comd_bag(&mut self) {
        if self.game.stash.is_empty() { self.push(" it's empty..?"); return; }
        let stash = self.game.stash.clone();
        for (i, w) in stash.iter().enumerate() {
            self.push_stylize(Line::from(vec![
                Span::raw(format!(" [{}]", i)),
                Span::styled(w.name.clone(), Style::default().fg(step_color(w.step))),
                Span::raw(format!("[Lv. {}] ['{}']", w.step, gems::element_name(w.mods.element))),
            ]));
        }
    }

    fn comd_runs(&mut self) {
        self.comd_runs_n(20);
    }

    fn comd_runs_n(&mut self, n: usize) {
        let idx = exist::load_index();
        if idx.is_empty() { self.push("no saved runs yet..."); return; }
        self.push(format!("current runs {}", n.min(idx.len())));
        for e in idx.iter().take(n) {
            self.push_stylize(Line::from(vec![
                Span::styled(format!(" {}", e.short_sig),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(format!(" stage {}  {}", e.stage, e.headline)),
            ]));
        }
        if idx.len() > n {
            self.push(format!(" ...and {} more... 'runs <number>' to show more, or type 'load <id>'",
                idx.len() - n));
        }
    }

    fn comd_load(&mut self, id: &str) {
        match exist::load(id) {
            Ok(g) => {
                self.game = g;
                self.next_spawn(tune::Rank::Normal);
                self.push_stylize(Line::from(vec![
                    Span::styled("Loaded & Checked", Style::default().fg(Color::Green)),
                    Span::raw(format!("run {id} - stage {}", self.game.stage)),
                ]));
            }
            Err(e) => self.push_stylize(Line::from(vec![
                Span::styled("Cannot load ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw(e),
            ])),
        }
    }

    fn comd_id(&mut self) {
        let id = id::loadout_short(&self.game.equipped);
        self.push_stylize(Line::from(vec![
            Span::raw(" char sheet id: "),
            Span::styled(id.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]));
        self.push(" share saves/run_<id>.json - others can import it and verify it works");
        for w in self.game.equipped.clone() {
            self.push(format!(" {} : {} ", id::short_sig(&w), w.name));
        }
    }

    fn comd_import(&mut self, path: &str) {
        match exist::import_file(path) {
            Ok(g) => { self.game = g; self.next_spawn(tune::Rank::Normal);
                self.push_stylize(Line::from(vec![
                    Span::styled("Read & Verified", Style::default().fg(Color::Green)),
                    Span::raw(format!("- stage {}", self.game.stage))
                ]));
            }
            Err(e) => self.push_stylize(Line::from(vec![
                Span::styled("Cannot read! ", Style::default().fg(Color::Red)), Span::raw(e)
            ])),
        }
    }

    fn comd_hb(&mut self) {

    }
}

fn step_color(step: u32) -> Color {
    match step {
        1..=2 => Color::Gray,
        3..=4 => Color::White,
        5..=6 => Color::Green,
        7..=8 => Color::Blue,
        9..=10 => Color::Magenta,
        _ => Color::Yellow,
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut sout = io::stdout(); // "cout"
    execute!(sout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(sout);
    let mut terminal = Terminal::new(backend)?;
    let mut board = Board::new();
    let mut save_timer = Instant::now();

    while !board.does_quit {
        board.tick();
        terminal.draw(|f| ui(f, &mut board))?;
        if poll(Duration::from_millis(100))? {
            if let Event::Key(k) = event::read()? {
                if k.kind == event::KeyEventKind::Press {
                    match k.code {
                        KeyCode::Enter => board.check_command(),
                        KeyCode::Char(c) => board.input.push(c),
                        KeyCode::Backspace => { board.input.pop(); }
                        //KeyCode::Esc => board.does_quit = true,
                        _ => {}
                    }
                }
            }
        }
        if save_timer.elapsed() > Duration::from_secs(11) {
            let _ = exist::save(&board.game);
            save_timer = Instant::now();
        }
    }

    let _ = exist::save(&board.game);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn ui(f: &mut Frame, board: &mut Board) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(1),
            Constraint::Length(3)])
        .split(f.area());

    let mut header_lines = vec![
        Line::from(format!(" Stage [ {} ] ..| Wealth [ {:.0}c ] ..| Kills [ {} ] ..| {}",
            board.game.stage, board.game.net_copper(), board.battles,
            if board.does_idle { "" } else { "WAIT" })),
        Line::from(format!("")),
        Line::from(format!(" {}", board.game.clock.date_string())),
        Line::from(format!("")),
        //Line::from(format!("{:?}", board.tick())),
    ];

    let enemy = board.at_enemy.clone();
    if let Some(e) = &enemy {
        let pct = (board.hp_enemy_remaining / e.hp_max * 100.0).clamp(0.0, 100.0);
        header_lines.push(Line::from(vec![
            Span::styled(format!(">< {}", e.name), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            //Span::from(format!("")),
            Span::raw(format!(" ({:.0}%) [+{}] <-- [{:.0}] dmg/s",
                pct,
                gems::element_name(e.resist_element),
                fight::effective_dps(&board.game.equipped, e))),
        ]));
        for (name, contrib, mult) in fight::breakdown(&board.game.equipped, e) {
            let tag = if mult < 0.5 { "..." } else if mult > 1.2 { "!!!" } else { "o" };
            header_lines.push(Line::from(format!(" {name}: {contrib:.0} dps (x{mult:.2} {tag})")));
        }
    }

    f.render_widget(
        Paragraph::new(header_lines).block(Block::default().borders(Borders::ALL).title("~epangelia~")),
        chunks[0]);
    let visible = chunks[1].height.saturating_sub(2) as usize;
    let start = board.log.len().saturating_sub(visible);
    let lines: Vec<Line> = board.log[start..].to_vec();
    f.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("~log~")),
        chunks[1]);

    f.render_widget(
        Paragraph::new(format!("> {}", board.input))
            .block(Block::default().borders(Borders::ALL).title("~input~")),
        chunks[2]);
}
