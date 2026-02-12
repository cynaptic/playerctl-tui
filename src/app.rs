use std::time::Duration;

use mpris::{LoopStatus, Metadata, PlaybackStatus, PlayerFinder};

pub struct App {
    pub running: bool,
    pub player_names: Vec<String>,
    pub selected_player: usize,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub position: Duration,
    pub duration: Duration,
    pub playback_status: String,
    pub volume: f64,
    pub loop_status: String,
    pub shuffle: bool,
    pub tick_count: u64,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            running: true,
            player_names: Vec::new(),
            selected_player: 0,
            title: String::new(),
            artist: String::new(),
            album: String::new(),
            position: Duration::ZERO,
            duration: Duration::ZERO,
            playback_status: String::from("Stopped"),
            volume: 0.0,
            loop_status: String::from("None"),
            shuffle: false,
            tick_count: 0,
        };
        app.refresh_players();
        app.refresh_state();
        app
    }

    pub fn refresh_players(&mut self) {
        let Ok(finder) = PlayerFinder::new() else {
            self.player_names.clear();
            return;
        };
        let Ok(players) = finder.find_all() else {
            self.player_names.clear();
            return;
        };
        let names: Vec<String> = players.iter().map(|p| p.identity().to_string()).collect();
        if names.is_empty() {
            self.player_names.clear();
            self.selected_player = 0;
            return;
        }
        // Preserve selection if possible
        let prev_name = self.player_names.get(self.selected_player).cloned();
        self.player_names = names;
        if let Some(prev) = prev_name {
            if let Some(idx) = self.player_names.iter().position(|n| n == &prev) {
                self.selected_player = idx;
            } else {
                self.selected_player = 0;
            }
        }
        if self.selected_player >= self.player_names.len() {
            self.selected_player = 0;
        }
    }

    pub fn refresh_state(&mut self) {
        let Some(player) = self.find_current_player() else {
            self.clear_track_info();
            return;
        };

        // Metadata
        let meta = player.get_metadata().unwrap_or_else(|_| Metadata::new(""));
        self.title = meta.title().unwrap_or("Unknown").to_string();
        self.artist = meta
            .artists()
            .map(|a| a.join(", "))
            .unwrap_or_else(|| "Unknown".to_string());
        self.album = meta.album_name().unwrap_or("Unknown").to_string();
        self.duration = meta.length().unwrap_or(Duration::ZERO);

        // Position
        self.position = player.get_position().unwrap_or(Duration::ZERO);

        // Playback status
        self.playback_status = match player.get_playback_status() {
            Ok(PlaybackStatus::Playing) => "Playing".to_string(),
            Ok(PlaybackStatus::Paused) => "Paused".to_string(),
            _ => "Stopped".to_string(),
        };

        // Volume
        self.volume = player.get_volume().unwrap_or(0.0);

        // Loop status
        self.loop_status = match player.get_loop_status() {
            Ok(LoopStatus::None) => "Off".to_string(),
            Ok(LoopStatus::Track) => "Track".to_string(),
            Ok(LoopStatus::Playlist) => "Playlist".to_string(),
            Err(_) => "N/A".to_string(),
        };

        // Shuffle
        self.shuffle = player.get_shuffle().unwrap_or(false);
    }

    fn clear_track_info(&mut self) {
        self.title.clear();
        self.artist.clear();
        self.album.clear();
        self.position = Duration::ZERO;
        self.duration = Duration::ZERO;
        self.playback_status = "Stopped".to_string();
        self.volume = 0.0;
        self.loop_status = "N/A".to_string();
        self.shuffle = false;
    }

    fn find_current_player(&self) -> Option<mpris::Player> {
        if self.player_names.is_empty() {
            return None;
        }
        let target = &self.player_names[self.selected_player];
        let finder = PlayerFinder::new().ok()?;
        let players = finder.find_all().ok()?;
        players.into_iter().find(|p| p.identity() == target)
    }

    pub fn toggle_play_pause(&self) {
        if let Some(player) = self.find_current_player() {
            let _ = player.play_pause();
        }
    }

    pub fn next_track(&self) {
        if let Some(player) = self.find_current_player() {
            let _ = player.next();
        }
    }

    pub fn prev_track(&self) {
        if let Some(player) = self.find_current_player() {
            let _ = player.previous();
        }
    }

    pub fn volume_up(&self) {
        if let Some(player) = self.find_current_player() {
            let vol = (self.volume + 0.05).min(1.0);
            let _ = player.set_volume(vol);
        }
    }

    pub fn volume_down(&self) {
        if let Some(player) = self.find_current_player() {
            let vol = (self.volume - 0.05).max(0.0);
            let _ = player.set_volume(vol);
        }
    }

    pub fn seek_forward(&self) {
        if let Some(player) = self.find_current_player() {
            let _ = player.seek_forwards(&Duration::from_secs(5));
        }
    }

    pub fn seek_backward(&self) {
        if let Some(player) = self.find_current_player() {
            let _ = player.seek_backwards(&Duration::from_secs(5));
        }
    }

    pub fn cycle_loop(&self) {
        if let Some(player) = self.find_current_player() {
            let next = match player.get_loop_status() {
                Ok(LoopStatus::None) => LoopStatus::Track,
                Ok(LoopStatus::Track) => LoopStatus::Playlist,
                Ok(LoopStatus::Playlist) => LoopStatus::None,
                Err(_) => return,
            };
            let _ = player.set_loop_status(next);
        }
    }

    pub fn toggle_shuffle(&self) {
        if let Some(player) = self.find_current_player() {
            let _ = player.set_shuffle(!self.shuffle);
        }
    }

    pub fn next_player(&mut self) {
        if !self.player_names.is_empty() {
            self.selected_player = (self.selected_player + 1) % self.player_names.len();
            self.refresh_state();
        }
    }

    pub fn prev_player(&mut self) {
        if !self.player_names.is_empty() {
            self.selected_player = if self.selected_player == 0 {
                self.player_names.len() - 1
            } else {
                self.selected_player - 1
            };
            self.refresh_state();
        }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
        if self.tick_count % 20 == 0 {
            self.refresh_players();
        }
        self.refresh_state();
    }
}
