//  Sand-Drop-Clicker
//  
//  Description:
//      A simple clicker game where you drop sand particles.
//
//  By:         Artem Suprun
//  Date:       11/03/2025
//  License:    Apache License 2.0
//  Github:     https://github.com/Artemsuprun/Sand-Drop-Clicker

// Needed imports
// standard library for data structures and time handling
use std::{
    collections::HashMap,
    collections::HashSet,
    time::Duration
};
// ggegui for GUI handling
use ggegui::{
    Gui,
    egui::{ self, Button }
};
// ggez for game framework
use ggez::{
	ContextBuilder, Context, GameResult,
	event::{ self, EventHandler }, 
	graphics::{ self, DrawParam, Color, Text, Rect, Mesh },
    input::keyboard::{ KeyCode, KeyMods, KeyInput }
};
// strum for enum iteration
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

// Global Variable
const FPS: u32 = 60; // Frames per second
const SCREEN_SIZE: (f32, f32) = (800.0, 600.0); // Screen dimensions
const GRAIN_SIZE: f32 = 20.0; // Size of each grain of sand
const GRAVITY: f32 = 300.0; // Gravity affecting the grains

// Set up and run the game
fn main() {
	let (mut ctx, event_loop) = ContextBuilder::new("SandDropClicker", "Artem Suprun")
        .window_setup(ggez::conf::WindowSetup::default().title("Sand Drop Clicker"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .unwrap();
	let state = SandDropClicker::new(&mut ctx);
	event::run(ctx, event_loop, state);
}

// Main game state
// holds the game logic and GUI
struct SandDropClicker {
    money: i64,
    particles: HashMap<SandParticle, Vec<Grain>>,
    upgrades: HashMap<Upgrade, u32>,
    total_clicks: u32,
    total_time: std::time::Duration,
    unlock: HashSet<Upgrade>,
    show_info: bool,
    autoclicker_timer: f32,
    gui: Gui,
    // needed for the graphics of the game: grains
    shared_mesh: Mesh,
}

impl SandDropClicker {
    // creates a new game state
    pub fn new(ctx: &mut Context) -> Self {
        // provide the game with the default upgrades
        let mut upgrades = HashMap::new();
        upgrades.insert(Upgrade::ParticleUpgrade, 1); // start with basic sand
        // create a shared mesh for the grains
        let rect = Rect::new(
            0.0,
            0.0,
            1.0,
            1.0
        );
        let square = Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            rect,
            Color::WHITE,
        ).unwrap();
        // create the game with default settings
        Self {
            money: 0,
            particles: HashMap::new(),
            upgrades: upgrades,
            total_clicks: 0,
            total_time: Duration::new(0, 0),
            unlock: HashSet::new(),
            show_info: false,
            autoclicker_timer: 0.0,
            gui: Gui::new(ctx),
            shared_mesh: square,
        }
    }

    // loads the options window
    fn options_gui(&mut self) {
        let gui_ctx = self.gui.ctx();
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
                        if !self.is_maxed(upgrade) {
                            let enabled: bool = self.money >= cost;
                            let btn_txt = format!("{}: {}$",upgrade.btn_txt(), cost);
                            ui.label(upgrade.desc());
                            if ui.add_enabled(enabled, Button::new(btn_txt)).clicked() {
                                self.buy(upgrade)
                            }
                        } else {
                            ui.label(format!("{}: MAXED", upgrade.btn_txt()));
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
            // check if gain can fit in container
            if current_amount + i >= container_size {
                break;
            }

            // add a sand particle at (x, y)
            let sand = self.rand_sand();
            let size = GRAIN_SIZE;
            let grain = Grain::new(x, y, size, sand);
            // Add the grain to the specific particle location.
            self.particles
                .entry(sand)
                .or_insert_with(Vec::new)
                .push(grain);
            
            i += 1;
        }
    }

    // simulates the autoclicker upgrade
    fn autoclicker(&mut self, seconds: f32) {
        // get the autoclicker level
        let autoclicker_level = *self.upgrades.get(&Upgrade::AutoClicker).unwrap_or(&0);
        if autoclicker_level > 0 && !self.is_full() {
            self.autoclicker_timer += seconds;
            let frequency = 1.0 / autoclicker_level as f32; // clicks per second

            let clicks = (self.autoclicker_timer / frequency).floor() as u32;
            for _ in 0..clicks {
                let x = rand::random::<f32>() * SCREEN_SIZE.0;
                let y = 0.0;
                self.add_grain(x, y);
                self.autoclicker_timer = 0.0;
            }
        }
    }

    // converts sand particles to money
    fn make_money(&mut self) {
        // sell all sand particles for money
        let mut earned = 0;
        for (particle, grains) in self.particles.iter_mut() {
            let count = grains.len() as i64;
            let value = particle.value();
            earned += count * value;
            grains.clear();
        }
        self.money += earned;
    }

    // returns true if the container is full
    fn is_full(&self) -> bool {
        // container size
        let size = self.get_size();
        let amount = self.get_amount();
        amount >= size
    }

    // returns the size of the container
    fn get_size(&self) -> u32{
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
        let mut amount = 0;
        for grains in self.particles.values() {
            amount += grains.len() as u32;
        }
        amount
    }

    // draws the game info on the screen
    fn game_info(&self, canvas: &mut graphics::Canvas) {
        let money = self.money;
        let size = self.get_size();
        let amount = self.get_amount();
        let txt = Text::new(
            format!("{}/{}\n{}$", amount, size, money)
        );
        canvas.draw(
            &txt,
            DrawParam::from([10.0, 10.0])
                .color(Color::WHITE),
        );
    }

    // draws the player info on the screen
    fn player_info(&self, canvas: &mut graphics::Canvas) { 
        let total_time = self.total_time.as_secs();
        let total_clicks = self.total_clicks;
        let txt = Text::new(
            format!("Total Time: {} seconds \nTotal Clicks: {}", 
                total_time, 
                total_clicks)
        );
        canvas.draw(
            &txt,
            DrawParam::from([10.0, 50.0])
                .color(Color::WHITE),
        );
    }

    // returns the cost of the upgrade
    fn upgrade_cost(&self, upgrade: Upgrade) -> i64 {
        let n = *self.upgrades.get(&upgrade).unwrap_or(&0);
        let cost: f64 = upgrade.cost(n);
        cost.round() as i64
    }

    // returns a random sand particle based on the current upgrade level
    fn rand_sand(&self) -> SandParticle {
        SandParticle::Sand
    }

    // buys the upgrade if the player has enough money
    fn buy(&mut self, upgrade: Upgrade) {
        let cost = self.upgrade_cost(upgrade);
        if self.money >= cost && !self.is_maxed(upgrade) {
            self.money -= cost;
            self.upgrades.entry(upgrade)
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

impl EventHandler for SandDropClicker {
    // update the game state
	fn update(&mut self, ctx: &mut Context) -> GameResult {
        // set up a fixed timestep for the physics of the grains
        while ctx.time.check_update_time(FPS) {
            let seconds = 1.0 / FPS as f32;
            // update the total_time stat
            self.total_time += Duration::from_secs_f32(seconds);

            // update the position of the falling particles.
            for grains in self.particles.values_mut() {
                for grain in grains.iter_mut() {
                    grain.update(seconds);
                }
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
		
        // draw the grain particle
        for grains in self.particles.values() {
            for grain in grains.iter() {
                canvas.draw(
                    &self.shared_mesh,
                    grain.draw_params()
                );
            }
        }

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
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeat: bool
    ) -> GameResult {
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

#[derive(Hash, Eq, PartialEq, Debug, EnumIter, Clone, Copy)]
enum Upgrade {
    BiggerContainer, // Adds more container space.
    ParticleUpgrade, // Provides more diverse sand particles, that differ in price.
    MoreParticles,   // Produce more sand particles per click.
    AutoClicker,     // Introduce an autoclicker, upgrades increase the clicking frequency.
}

impl Upgrade {
    // Button text for the upgrade
    fn btn_txt(&self) -> &str {
        match self {
            Upgrade::BiggerContainer => "Buy Bigger Container",
            Upgrade::ParticleUpgrade => "Improve Sand Quality",
            Upgrade::MoreParticles =>   "Buy More Particles",
            Upgrade::AutoClicker =>     "Buy Auto Clicker",
        }
    }

    // Description for the upgrade
    fn desc(&self) -> &str {
        match self {
            Upgrade::BiggerContainer => "This will increase your container size:",
            Upgrade::ParticleUpgrade => "This will allow you a chances to drop better sand:",
            Upgrade::MoreParticles =>   "This will allow you to drop more sand per click:",
            Upgrade::AutoClicker =>     "This will drop sand for you:",
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
            Upgrade::ParticleUpgrade => SandParticle::cost(n) as f64,
            Upgrade::MoreParticles =>   200.0,
            Upgrade::AutoClicker =>     500.0,
        };

        if *self == Upgrade::ParticleUpgrade {
            base_cost
        } else {
            base_cost * base_m.powf(m)
        }
    }

    // returns the maximum level of the upgrade, if any
    fn max_level(&self) -> Option<u32> {
        match self {
            Upgrade::ParticleUpgrade => Some(SandParticle::max_level()),
            Upgrade::AutoClicker =>     Some(500),
            _ => None, // no limit for other upgrades
        }
    }
}

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

impl SandParticle {
    // returns the value of the sand particle
    fn value(&self) -> i64 {
        match self {
            SandParticle::Sand =>       1,
            SandParticle::Quartz =>     1,
            SandParticle::Shell =>      1,
            SandParticle::Coral =>      1,
            SandParticle::Pinksand =>   1,
            SandParticle::Volcanic =>   1,
            SandParticle::Glauconite => 1,
            SandParticle::Gemstones =>  1,
            SandParticle::Iron =>       1,
            SandParticle::Starsand =>   1,
            SandParticle::Gold =>       1,
            SandParticle::Diamond =>    1,
        }
    }

    // returns the color of the sand particle
    fn color(&self) -> Color {
        match self {
            SandParticle::Sand =>       Color::from_rgb(243, 213, 103),
            SandParticle::Quartz =>     Color::from_rgb(243, 213, 103),
            SandParticle::Shell =>      Color::from_rgb(243, 213, 103),
            SandParticle::Coral =>      Color::from_rgb(243, 213, 103),
            SandParticle::Pinksand =>   Color::from_rgb(243, 213, 103),
            SandParticle::Volcanic =>   Color::from_rgb(243, 213, 103),
            SandParticle::Glauconite => Color::from_rgb(243, 213, 103),
            SandParticle::Gemstones =>  Color::from_rgb(243, 213, 103),
            SandParticle::Iron =>       Color::from_rgb(243, 213, 103),
            SandParticle::Starsand =>   Color::from_rgb(243, 213, 103),
            SandParticle::Gold =>       Color::from_rgb(243, 213, 103),
            SandParticle::Diamond =>    Color::from_rgb(243, 213, 103),
        }
    }

    // returns the cost of the sand particle based on its level
    fn cost(num: u32) -> i64 {
        let particle = SandParticle::from_u32(num).unwrap();
        match particle {
            SandParticle::Sand =>       0,
            SandParticle::Quartz =>     100,
            SandParticle::Shell =>      300,
            SandParticle::Coral =>      1000,
            SandParticle::Pinksand =>   2000,
            SandParticle::Volcanic =>   6000,
            SandParticle::Glauconite => 12000,
            SandParticle::Gemstones =>  24000,
            SandParticle::Iron =>       50000,
            SandParticle::Starsand =>   100000,
            SandParticle::Gold =>       1000000,
            SandParticle::Diamond =>    10000000,
        }
    }

    // returns the sand particle from its level number
    fn from_u32(num: u32) -> Option<Self> {
        match num {
            0 =>  Some(SandParticle::Sand),
            1 =>  Some(SandParticle::Quartz),
            2 =>  Some(SandParticle::Shell),
            3 =>  Some(SandParticle::Coral),
            4 =>  Some(SandParticle::Pinksand),
            5 =>  Some(SandParticle::Volcanic),
            6 =>  Some(SandParticle::Glauconite),
            7 =>  Some(SandParticle::Gemstones),
            8 =>  Some(SandParticle::Iron),
            9 =>  Some(SandParticle::Starsand),
            10 => Some(SandParticle::Gold),
            11 => Some(SandParticle::Diamond),
            _  => None,
        }
    }

    // returns the maximum level of sand particles
    fn max_level() -> u32 {
        SandParticle::iter().count() as u32
    }
}

#[derive(Debug)]
struct Grain {
    rect: Rect,
    rotation: f32,
    color: Color,
    x_v: f32,
    y_v: f32,
    x_a: f32,
    y_a: f32,
}

impl Grain {
    // creates a new grain of sand
    fn new(x: f32, y: f32, size: f32, sand: SandParticle) -> Self {
        let color = sand.color();
        let rect = Rect::new(
            x - size / 2.0,
            y - size / 2.0,
            size,
            size
        );

        Self {
            rect: rect,
            rotation: 0.0,
            color: color,
            x_v: 0.0,
            y_v: 0.0,
            x_a: 0.0,
            y_a: GRAVITY,
        }
    }

    // returns the center point of the grain
    fn center(&self) -> [f32;2] {
        [
            self.rect.x + self.rect.w / 2.0,
            self.rect.y + self.rect.h / 2.0
        ]
    }

    // returns the top-left point of the grain
    fn point(&self) -> [f32;2] {
        [
            self.rect.x,
            self.rect.y
        ]
    }

    // returns the size of the grain
    fn size(&self) -> [f32;2] {
        [
            self.rect.w,
            self.rect.h
        ]
    }

    // updates the position of the grain based on physics
    fn update(&mut self, dt: f32) {
        // apply gravity to acceleration, it's a constant
        self.y_a = GRAVITY;
        // update velocity based on acceleration
        self.y_v += self.y_a * dt;
        // move the grain downwards
        self.rect.translate([0.0, self.y_v * dt]);

        if self.rect.bottom() >= SCREEN_SIZE.1 {
            self.rect.y = SCREEN_SIZE.1 - self.rect.h;
            self.y_v = 0.0;
        }
    }

    // returns the draw parameters for the grain
    fn draw_params(&self) -> DrawParam {
        DrawParam::default()
            .dest(self.center())
            .rotation(self.rotation)
            .scale(self.size())
            .offset([0.5, 0.5])
            .color(self.color)
    }
}
