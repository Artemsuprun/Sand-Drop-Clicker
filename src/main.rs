//  Sand-Drop-Clicker
//
//  Description:
//      A simple clicker game where you drop sand particles.
//
//  By:         Artem Suprun
//  Date:       12/09/2025
//  License:    Apache License 2.0
//  Github:     https://github.com/Artemsuprun/Sand-Drop-Clicker

//! # Sand Drop Clicker
//! A simple clicker game where you drop sand particles by clicking
//! on the screen. You can earn money by converting sand particles
//! and use that money to buy upgrades that enhance your sand dropping
//! capabilities. The game features different types of sand particles,
//! each with its own value, and various upgrades to improve your
//! sand dropping efficiency.

//! ## Controls:
//! - Click anywhere on the screen to drop sand particles.
//! - Press `Ctrl + I` to toggle the display of player information.
//! - Press `Ctrl + Q` to quit the game.

//! ## Needed Crates:
//! - ggez: Game framework for Rust.
//! - ggegui: GUI library for ggez.
//! - rand: Random number generation.
//! - strum: Enum iteration utilities.
//! - strum_macros: Macros for strum.

// Needed imports
// standard library for data structures and time handling
use std::{collections::HashMap, collections::HashSet, time::Duration};
// rand for random number generation
use rand::Rng;
// ggegui for GUI handling
use ggegui::{
    Gui,
    egui::{self, Button},
};
// ggez for game framework
use ggez::{
    Context, ContextBuilder, GameResult,
    event::{self, EventHandler},
    graphics::{self, Color, DrawParam, Image, InstanceArray, Rect, Text},
    input::keyboard::{KeyCode, KeyInput, KeyMods},
};
// strum for enum iteration
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

// Global Variable
const FPS: u32 = 30; // Frames per second
const SCREEN_SIZE: (f32, f32) = (800.0, 600.0); // Screen dimensions
const GRAIN_SIZE: f32 = 10.0; // Size of each grain of sand
const GRAVITY: f32 = 300.0; // Gravity affecting the grains

// Set up and run the game
fn main() {
    // create the ggez context and event loop
    let (mut ctx, event_loop) = ContextBuilder::new("SandDropClicker", "Artem Suprun")
        .window_setup(ggez::conf::WindowSetup::default().title("Sand Drop Clicker"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .unwrap();
    // create the game state
    let state = SandDropClicker::new(&mut ctx);
    // run the game
    event::run(ctx, event_loop, state);
}

// Main game state
// holds the game logic and GUI
/// game state structure
/// * money: player's current money
/// * particles: map of sand particles and their counts
/// * grains: vector of grain instances
/// * upgrades: map of upgrades and their levels
/// * total_clicks: total number of clicks made by the player
/// * total_time: total time spent in the game
/// * unlock: set of unlocked upgrades
/// * show_info: flag to show/hide player info
/// * autoclicker_timer: timer for the autoclicker upgrade
/// * gui: GUI instance for the game
/// * batch: instance array for rendering grains
struct SandDropClicker {
    money: i64,
    particles: HashMap<SandParticle, u32>,
    grains: Vec<Grain>,
    upgrades: HashMap<Upgrade, u32>,
    total_clicks: u32,
    total_time: std::time::Duration,
    unlock: HashSet<Upgrade>,
    show_info: bool,
    autoclicker_timer: f32,
    gui: Gui,
    // needed for the graphics of the game: grains
    batch: InstanceArray,
}

/// Implementation of the game logic and GUI handling
/// for the SandDropClicker struct
/// Contains methods for game initialization, GUI updates,
/// sand particle management, upgrades, and event handling.
impl SandDropClicker {
    // creates a new game state
    pub fn new(ctx: &mut Context) -> Self {
        // provide the game with the default upgrades
        let mut upgrades_map = HashMap::new();
        upgrades_map.insert(Upgrade::ParticleTier, 1); // start with basic sand
        // create a shared mesh for the grains
        let square = Image::from_color(ctx, 1, 1, Some(Color::WHITE));
        let batch_array = InstanceArray::new(ctx, square);
        // create the game with default settings
        Self {
            money: 0,
            particles: HashMap::new(),
            grains: Vec::new(),
            upgrades: upgrades_map,
            total_clicks: 0,
            total_time: Duration::new(0, 0),
            unlock: HashSet::new(),
            show_info: false,
            autoclicker_timer: 0.0,
            gui: Gui::new(ctx),
            batch: batch_array,
        }
    }

    // loads the options window
    fn options_gui(&mut self) {
        // get the GUI context
        let gui_ctx = self.gui.ctx();
        // create the options window
        egui::Window::new("Options")
            .resizable(false)
            .default_size([250.0, 100.0])
            .default_pos([10.0, 100.0])
            .show(&gui_ctx, |ui| {
                // Display instructions
                ui.label("Click the button to earn money!");
                if ui.button("Convert").clicked() {
                    self.make_money();
                }
                // display money
                ui.label(format!("Money: {}$", self.money));

                // show available upgrades
                ui.separator();
                if self.unlock.is_empty() {
                    ui.label("No upgrades available yet. Keep clicking!");
                } else {
                    ui.label("Available Upgrades:");
                }
                for upgrade in Upgrade::iter() {
                    let cost = self.upgrade_cost(upgrade);
                    if self.unlock.contains(&upgrade) {
                        ui.label(upgrade.desc());
                        let amount = *self.upgrades.get(&upgrade).unwrap_or(&0);
                        if !self.is_maxed(upgrade) {
                            let enabled: bool = self.money >= cost;
                            let btn_txt = format!("{} ({}): {}$", upgrade.btn_txt(), amount, cost);
                            if ui.add_enabled(enabled, Button::new(btn_txt)).clicked() {
                                self.buy(upgrade)
                            }
                        } else {
                            let btn_txt =
                                format!("{} ({}): (MAX LEVEL)", upgrade.btn_txt(), amount);
                            ui.add_enabled(false, Button::new(btn_txt));
                        }
                    } else if self.money >= cost {
                        self.unlock.insert(upgrade);
                    }
                }
            });
    }

    // simulates dropping a sand particle
    fn add_grain(&mut self, x: f32, y: f32) {
        // for multiple grains spawning
        let amount = 1 + *self.upgrades.get(&Upgrade::MoreParticles).unwrap_or(&0);
        // variable to track how many grains have been added
        let mut i: u32 = 0;
        let container_size = self.get_size();
        let current_amount = self.get_amount();
        while i < amount {
            let mut new_x = x;
            let mut new_y = y;
            // add slight random offset for multiple grains
            if i > 0 {
                let max_offset = 50.0;
                let offset_x = rand::rng().random_range(-max_offset..max_offset);
                let offset_y = rand::rng().random_range(-max_offset..max_offset);
                new_x = (x + offset_x).clamp(0.0, SCREEN_SIZE.0);
                new_y = y + offset_y;
            }

            // check if gain can fit in container
            if current_amount + i >= container_size {
                break;
            }

            // add a sand particle at (x, y)
            let sand = self.rand_sand();
            let size = GRAIN_SIZE;
            let grain = Grain::new(new_x, new_y, size, sand.color());
            // Add the grain to the specific particle location.
            self.particles
                .entry(sand)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            self.grains.push(grain);

            i += 1;
        }
    }

    // simulates the autoclicker upgrade
    fn autoclicker(&mut self, seconds: f32) {
        // get the autoclicker level
        let autoclicker_level = *self.upgrades.get(&Upgrade::AutoClicker).unwrap_or(&0);
        if autoclicker_level > 0 && !self.is_full() {
            // increment the timer
            self.autoclicker_timer += seconds;
            let frequency = 5.0 / autoclicker_level as f32; // clicks per second
            // determine how many clicks to make
            let clicks = (self.autoclicker_timer / frequency).floor() as u32;
            for _ in 0..clicks {
                let x = rand::random::<f32>() * SCREEN_SIZE.0;
                let y = 0.0;
                self.add_grain(x, y);
                // reset the timer
                self.autoclicker_timer = 0.0;
            }
        }
    }

    // converts sand particles to money
    fn make_money(&mut self) {
        // sell all sand particles for money
        let mut earned = 0;
        for (particle, count) in self.particles.iter_mut() {
            let value = particle.value();
            earned += (*count as i64) * value;
            // reset the count of the particle
            *count = 0;
        }
        self.money += earned;
        // clear the grains vector
        self.grains.clear();
    }

    // returns true if the container is full
    fn is_full(&self) -> bool {
        // container size
        let size = self.get_size();
        let amount = self.get_amount();
        amount >= size
    }

    // returns the size of the container
    fn get_size(&self) -> u32 {
        // base container size
        let base_size = 25;
        // amount of upgrades for bigger container.
        let upgrade = 1 + *self.upgrades.get(&Upgrade::BiggerContainer).unwrap_or(&0);
        // calculate the total size
        base_size * upgrade
    }

    // returns the amount of particles in the container
    fn get_amount(&self) -> u32 {
        // count the amount of particles in the container
        self.grains.len() as u32
    }

    // draws the game info on the screen
    fn game_info(&self, canvas: &mut graphics::Canvas) {
        let money = self.money;
        let size = self.get_size();
        let amount = self.get_amount();
        let txt = Text::new(format!("{}/{}\n{}$", amount, size, money));
        canvas.draw(&txt, DrawParam::from([10.0, 10.0]).color(Color::WHITE));
    }

    // draws the player info on the screen
    fn player_info(&self, canvas: &mut graphics::Canvas) {
        let total_time = self.total_time.as_secs();
        let total_clicks = self.total_clicks;
        let txt = Text::new(format!(
            "Total Time: {} seconds \nTotal Clicks: {}",
            total_time, total_clicks
        ));
        canvas.draw(&txt, DrawParam::from([10.0, 50.0]).color(Color::WHITE));
    }

    // returns the cost of the upgrade
    fn upgrade_cost(&self, upgrade: Upgrade) -> i64 {
        let n = *self.upgrades.get(&upgrade).unwrap_or(&0);
        let cost: f64 = upgrade.cost(n);
        cost.round() as i64
    }

    // returns a random sand particle based on the current upgrade level
    fn rand_sand(&self) -> SandParticle {
        let level = *self.upgrades.get(&Upgrade::ParticleTier).unwrap_or(&0);
        let sand_level = rand::random::<u32>() % (level);
        SandParticle::from_u32(sand_level).unwrap_or(SandParticle::Sand)
    }

    // buys the upgrade if the player has enough money
    fn buy(&mut self, upgrade: Upgrade) {
        let cost = self.upgrade_cost(upgrade);
        if self.money >= cost && !self.is_maxed(upgrade) {
            self.money -= cost;
            self.upgrades
                .entry(upgrade)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    // returns true if the upgrade is maxed out
    fn is_maxed(&self, upgrade: Upgrade) -> bool {
        match upgrade.max_level() {
            Some(max) => {
                let current = *self.upgrades.get(&upgrade).unwrap_or(&0);
                current >= max
            }
            None => false,
        }
    }
}

/// Event handling for the SandDropClicker game
/// Implements the ggez EventHandler trait
/// to handle game updates, drawing, mouse clicks, and key events.
impl EventHandler for SandDropClicker {
    // update the game state
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // set up a fixed timestep for the physics of the grains
        while ctx.time.check_update_time(FPS) {
            let seconds = 1.0 / FPS as f32;
            // update the total_time stat
            self.total_time += Duration::from_secs_f32(seconds);

            // update the position of the falling particles.
            for grain in &mut self.grains {
                // skip updating if the grain is done
                if grain.is_done() {
                    continue;
                }
                grain.update(seconds);
            }

            // autoclicker upgrade
            self.autoclicker(seconds);

            // TODO: collision between grains
        }

        // update the GUI
        self.options_gui();
        self.gui.update(ctx);
        Ok(())
    }

    // draw the game state
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // clear the screen
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);

        // draw the grain particles
        self.batch.clear();
        if self.batch.capacity() < self.grains.len() {
            self.batch.resize(ctx, self.grains.len());
        }
        for grain in &self.grains {
            // skip drawing if the grain is done
            if grain.is_done() {
                continue;
            }
            self.batch.push(grain.draw_params());
        }
        canvas.draw(&self.batch, DrawParam::default());

        // draw the player stat
        self.game_info(&mut canvas);

        // draw the gui
        canvas.draw(&self.gui, DrawParam::default());

        // draw game info
        if self.show_info {
            self.player_info(&mut canvas);
        }

        // finish drawing
        canvas.finish(ctx).unwrap();
        Ok(())
    }

    // handle mouse clicks
    // if the pointer is over the GUI, ignore the click
    // otherwise, drop a grain of sand.
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        // Ignore clicks if the pointer is over the GUI or the container is full
        if !self.gui.ctx().wants_pointer_input() && !self.is_full() {
            // increment total clicks
            self.total_clicks += 1;
            self.add_grain(x, y);
        }
        Ok(())
    }

    // handle key down events
    // Ctrl+I to toggle info display
    // Ctrl+Q to quit the game
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            Some(KeyCode::I) => {
                if input.mods.contains(KeyMods::CTRL) {
                    self.show_info = !self.show_info;
                }
            }
            Some(KeyCode::Q) => {
                if input.mods.contains(KeyMods::CTRL) {
                    ctx.request_quit();
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Different types of upgrades available in the game
/// * BiggerContainer: Increases container size.
/// * ParticleTier: Unlocks better sand particles.
/// * AutoClicker: Automatically drops sand particles.
/// * MoreParticles: Increases number of particles dropped per click.
#[derive(Hash, Eq, PartialEq, Debug, EnumIter, Clone, Copy)]
enum Upgrade {
    BiggerContainer, // Adds more container space.
    ParticleTier,    // Provides more diverse sand particles, that differ in price.
    AutoClicker,     // Introduce an autoclicker, upgrades increase the clicking frequency.
    MoreParticles,   // Produce more sand particles per click.
}

/// Implementation of methods for the Upgrade enum
/// * btn_txt: returns the button text for the upgrade
/// * desc: returns the description of the upgrade
/// * cost: returns the cost of the upgrade based on its current level
/// * max_level: returns the maximum level of the upgrade, if any
impl Upgrade {
    // Button text for the upgrade
    fn btn_txt(&self) -> &str {
        match self {
            Upgrade::BiggerContainer => "Buy Bigger Container",
            Upgrade::ParticleTier => "Improve Sand Quality",
            Upgrade::AutoClicker => "Buy Auto Clicker",
            Upgrade::MoreParticles => "Buy More Particles",
        }
    }

    // Description for the upgrade
    fn desc(&self) -> &str {
        match self {
            Upgrade::BiggerContainer => "This will increase your container size:",
            Upgrade::ParticleTier => "This will allow you a chances to drop better sand:",
            Upgrade::AutoClicker => "This will drop sand for you:",
            Upgrade::MoreParticles => "This will allow you to drop more sand per click:",
        }
    }

    // returns the cost of the upgrade based on the current level
    fn cost(&self, n: u32) -> f64 {
        // formula: upgrade_base_cost * 1.1^m
        let m: f64 = n as f64;
        let base_m: f64 = 1.1;

        // get the base cost depending on the upgrade type
        let base_cost: f64 = match self {
            Upgrade::BiggerContainer => 50.0,
            Upgrade::ParticleTier => SandParticle::cost(n) as f64,
            Upgrade::AutoClicker => 700.0,
            Upgrade::MoreParticles => 1000.0,
        };

        if *self == Upgrade::ParticleTier {
            base_cost
        } else {
            base_cost * base_m.powf(m)
        }
    }

    // returns the maximum level of the upgrade, if any
    fn max_level(&self) -> Option<u32> {
        match self {
            Upgrade::ParticleTier => Some(SandParticle::max_level()),
            Upgrade::AutoClicker => Some(100),
            Upgrade::MoreParticles => Some(50),
            _ => None, // no limit for other upgrades
        }
    }
}

/// Different types of sand particles available in the game
#[derive(Hash, Eq, PartialEq, Debug, EnumIter, Clone, Copy)]
enum SandParticle {
    Sand,
    Quartz,
    Shell,
    Coral,
    Pinksand,
    Volcanic,
    Glauconite,
    Gemstones,
    Iron,
    Starsand,
    Gold,
    Diamond,
}

/// Implementation of methods for the SandParticle enum
/// * value: returns the value of the sand particle
/// * color: returns the color of the sand particle
/// * cost: returns the cost of the sand particle based on its level
/// * from_u32: returns the sand particle from its level number
/// * max_level: returns the maximum level of sand particles
impl SandParticle {
    // returns the value of the sand particle
    fn value(&self) -> i64 {
        match self {
            SandParticle::Sand => 1,
            SandParticle::Quartz => 2,
            SandParticle::Shell => 4,
            SandParticle::Coral => 8,
            SandParticle::Pinksand => 16,
            SandParticle::Volcanic => 32,
            SandParticle::Glauconite => 64,
            SandParticle::Gemstones => 128,
            SandParticle::Iron => 256,
            SandParticle::Starsand => 512,
            SandParticle::Gold => 1024,
            SandParticle::Diamond => 2048,
        }
    }

    // returns the color of the sand particle
    fn color(&self) -> Color {
        match self {
            SandParticle::Sand => Color::from_rgb(243, 213, 103),
            SandParticle::Quartz => Color::from_rgb(169, 170, 171),
            SandParticle::Shell => Color::from_rgb(255, 241, 231),
            SandParticle::Coral => Color::from_rgb(248, 131, 121),
            SandParticle::Pinksand => Color::from_rgb(246, 196, 193),
            SandParticle::Volcanic => Color::from_rgb(162, 151, 158),
            SandParticle::Glauconite => Color::from_rgb(46, 111, 64),
            SandParticle::Gemstones => Color::from_rgb(153, 102, 204),
            SandParticle::Iron => Color::from_rgb(133, 81, 65),
            SandParticle::Starsand => Color::from_rgb(255, 250, 134),
            SandParticle::Gold => Color::from_rgb(211, 175, 55),
            SandParticle::Diamond => Color::from_rgb(154, 197, 219),
        }
    }

    // returns the cost of the sand particle based on its level
    fn cost(num: u32) -> i64 {
        let particle = SandParticle::from_u32(num);
        match particle {
            Some(particle) => match particle {
                SandParticle::Sand => 0,
                SandParticle::Quartz => 100,
                SandParticle::Shell => 500,
                SandParticle::Coral => 2000,
                SandParticle::Pinksand => 8000,
                SandParticle::Volcanic => 10000,
                SandParticle::Glauconite => 50000,
                SandParticle::Gemstones => 100000,
                SandParticle::Iron => 500000,
                SandParticle::Starsand => 1000000,
                SandParticle::Gold => 5000000,
                SandParticle::Diamond => 10000000,
            },
            None => 0,
        }
    }

    // returns the sand particle from its level number
    fn from_u32(num: u32) -> Option<Self> {
        match num {
            0 => Some(SandParticle::Sand),
            1 => Some(SandParticle::Quartz),
            2 => Some(SandParticle::Shell),
            3 => Some(SandParticle::Coral),
            4 => Some(SandParticle::Pinksand),
            5 => Some(SandParticle::Volcanic),
            6 => Some(SandParticle::Glauconite),
            7 => Some(SandParticle::Gemstones),
            8 => Some(SandParticle::Iron),
            9 => Some(SandParticle::Starsand),
            10 => Some(SandParticle::Gold),
            11 => Some(SandParticle::Diamond),
            _ => None,
        }
    }

    // returns the maximum level of sand particles
    fn max_level() -> u32 {
        SandParticle::iter().count() as u32
    }
}

/// Structure representing a grain of sand
/// * rect: rectangle representing the grain's position and size
/// * color: color of the grain
/// * rotation: current rotation of the grain
/// * r_v: rotational velocity of the grain
/// * y_v: vertical velocity of the grain
/// * y_a: vertical acceleration of the grain
#[derive(Debug)]
struct Grain {
    rect: Rect,
    color: Color,
    rotation: f32,
    r_v: f32,
    y_v: f32,
    y_a: f32,
}

/// Implementation of methods for the Grain struct
/// * new: creates a new grain of sand
/// * is_done: returns true if the grain is done (on the ground)
/// * update: updates the position of the grain based on physics
/// * draw_params: returns the draw parameters for the grain
impl Grain {
    // creates a new grain of sand
    fn new(x: f32, y: f32, size: f32, rgb: Color) -> Self {
        let grain_rect = Rect::new(x - size / 2.0, y - size / 2.0, size, size);

        Self {
            rect: grain_rect,
            color: rgb,
            rotation: 0.0,
            r_v: 3.0,
            y_v: 0.0,
            y_a: 0.0,
        }
    }

    // returns true if the grain is done (on the ground)
    fn is_done(&self) -> bool {
        self.rect.bottom() >= SCREEN_SIZE.1 && self.y_v <= 0.1
    }

    // updates the position of the grain based on physics
    fn update(&mut self, dt: f32) {
        // put the physics to sleep if on the ground
        if self.is_done() {
            return;
        }
        // apply gravity
        self.y_v += GRAVITY * dt;
        // apply acceleration
        self.y_v += self.y_a * dt;
        // update position based on velocity
        self.rect.translate([0.0, self.y_v * dt]);
        self.rotation += self.r_v * dt;
        // check for ground collision
        if self.rect.bottom() >= SCREEN_SIZE.1 {
            self.rect.y = SCREEN_SIZE.1 - self.rect.h;
            self.y_v = 0.0;
        }
    }

    // returns the draw parameters for the grain
    fn draw_params(&self) -> DrawParam {
        DrawParam::default()
            .dest(self.rect.center())
            .rotation(self.rotation)
            .scale(self.rect.size())
            .offset([0.5, 0.5])
            .color(self.color)
    }
}

/// Tests for SandDropClicker
/// Contains unit tests for various components of the game.
#[cfg(test)]
mod tests {
    use super::*;
    
    // Upgrade tests
    #[test]
    fn test_upgrade_desc() {
        let upgrade = Upgrade::MoreParticles;
        assert_eq!(upgrade.desc(), "This will allow you to drop more sand per click:");
    }
    #[test]
    fn test_upgrade_btn_txt() {
        let upgrade = Upgrade::AutoClicker;
        assert_eq!(upgrade.btn_txt(), "Buy Auto Clicker");
    }
    #[test]
    fn test_upgrade_cost() {
        let upgrade = Upgrade::BiggerContainer;
        let base_m: f64 = 1.1;
        let base_cost: f64 = 50.0;
        let m: f64 = 100.0;
        let cost_level_100 = base_cost * base_m.powf(m);
        assert_eq!(upgrade.cost(0), 50.0);
        assert_eq!(upgrade.cost(100), cost_level_100);
    }
    #[test]
    fn test_upgrade_max_level() {
        let upgrade = Upgrade::ParticleTier;
        assert_eq!(upgrade.max_level(), Some(SandParticle::max_level()));
    }

    // SandParticle tests
    #[test]
    fn test_sand_particle_color() {
        let particle = SandParticle::Coral;
        assert_eq!(particle.color(), Color::from_rgb(248, 131, 121));
    }
    #[test]
    fn test_sand_particle_value() {
        let particle = SandParticle::Gold;
        assert_eq!(particle.value(), 1024);
    }
    #[test]
    fn test_sand_particle_cost() {
        assert_eq!(SandParticle::cost(0), 0);
        assert_eq!(SandParticle::cost(1), 100);
        assert_eq!(SandParticle::cost(11), 10000000);
    }
    #[test]
    fn test_sand_particle_from_u32() {
        assert_eq!(SandParticle::from_u32(0), Some(SandParticle::Sand));
        assert_eq!(SandParticle::from_u32(5), Some(SandParticle::Volcanic));
        assert_eq!(SandParticle::from_u32(12), None);
    }
    #[test]
    fn test_sand_particle_max_level() {
        assert_eq!(SandParticle::max_level(), 12);
    }

    // Grain tests
    #[test]
    fn test_grain_new() {
        let grain = Grain::new(100.0, 200.0, GRAIN_SIZE, Color::WHITE);
        assert_eq!(grain.rect.x, 100.0 - GRAIN_SIZE / 2.0);
        assert_eq!(grain.rect.y, 200.0 - GRAIN_SIZE / 2.0);
        assert_eq!(grain.rect.w, GRAIN_SIZE);
        assert_eq!(grain.rect.h, GRAIN_SIZE);
        assert_eq!(grain.color, Color::WHITE);
    }
    #[test]
    fn test_grain_is_done() {
        let grain = Grain::new(0.0, SCREEN_SIZE.1 + 10.0, GRAIN_SIZE, Color::WHITE);
        assert!(grain.is_done());
    }
    #[test]
    fn test_grain_update() {
        let mut grain = Grain::new(0.0, 0.0, GRAIN_SIZE, Color::WHITE);
        grain.update(1.0);
        assert!(grain.rect.y > 0.0);
    }
}