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
use std::{
    collections::HashMap,
    collections::HashSet,
    time::Duration
};
use ggegui::{
    Gui,
    egui::{ self, Button }
};
use ggez::{
	ContextBuilder, Context, GameResult,
	event::{ self, EventHandler }, 
	graphics::{ self, DrawParam, Color, Text, Rect, Mesh },
    input::keyboard::{ KeyCode, KeyMods, KeyInput }
};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

// Global Variable
const FPS: u32 = 60;
const SCREEN_SIZE: (f32, f32) = (800.0, 600.0);
const GRAIN_SIZE: f32 = 20.0;

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
    // needed for the GUI
    gui: Gui,
    // needed for the graphics of the game: grains
    shared_mesh: Mesh,
}

impl SandDropClicker {
    pub fn new(ctx: &mut Context) -> Self {
        // provide the game with the default upgrades
        let mut upgrades = HashMap::new();
        upgrades.insert(Upgrade::ParticleUpgrade, 1);
        upgrades.insert(Upgrade::BiggerContainer, 1);
        upgrades.insert(Upgrade::MoreParticles, 1);
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
            gui: Gui::new(ctx),
            unlock: HashSet::new(),
            show_info: false,
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
                        let mut enabled: bool = !self.is_maxed(upgrade);
                        enabled = enabled && (self.money >= cost);
                        let btn_txt = format!("{}: {}$",upgrade.btn_txt(), cost);
                        ui.label(upgrade.desc());
                        if ui.add_enabled(enabled, Button::new(btn_txt)).clicked() {
                            self.buy(upgrade)
                        }
                    } else if self.money >= cost {
                        self.unlock.insert(upgrade);
                    }
                }
		    });
    }

    // simulates dropping a sand particle
    fn add_grain(&mut self, x: f32, y: f32) {
        // increment total clicks
        self.total_clicks += 1;

        // for multiple grains spawning
        let amount = *self.upgrades.get(&Upgrade::MoreParticles).unwrap_or(&1);
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

    fn is_full(&self) -> bool {
        // container size
        let size = self.get_size();
        let amount = self.get_amount();
        amount >= size
    }

    fn get_size(&self) -> u32{
        // base container size
        let base_size = 25;
        // amount of upgrades for bigger container.
        let upgrade = *self.upgrades.get(&Upgrade::BiggerContainer).unwrap_or(&1);

        base_size * upgrade
    }

    fn get_amount(&self) -> u32 {
        // count the amount of particles in the container
        let mut amount = 0;
        for grains in self.particles.values() {
            amount += grains.len() as u32;
        }
        amount
    }

    fn game_info(&self, canvas: &mut graphics::Canvas) {
        let money = self.money;
        let size = self.get_size();
        let amount = self.get_amount();
        let txt = Text::new(
            format!("{}/{}\n{}$", amount, size, money)
        );
        canvas.draw(
            &txt,
            graphics::DrawParam::from([10.0, 10.0])
                .color(Color::WHITE),
        );
    }

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
            graphics::DrawParam::from([10.0, 50.0])
                .color(Color::WHITE),
        );
    }

    fn upgrade_cost(&self, upgrade: Upgrade) -> i64 {
        let n = *self.upgrades.get(&upgrade).unwrap_or(&0);
        let cost: f64 = upgrade.cost(n);
        cost.round() as i64
    }

    fn rand_sand(&self) -> SandParticle {
        SandParticle::Sand
    }

    fn buy(&mut self, upgrade: Upgrade) {
        let cost = self.upgrade_cost(upgrade);
        if self.money >= cost {
            self.money -= cost;
            self.upgrades.entry(upgrade)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    // returns the boolean result
    fn is_maxed(&self, upgrade: Upgrade) -> bool {
        if upgrade == Upgrade::ParticleUpgrade {
            match self.upgrades.get(&upgrade) {
                Some(num) => SandParticle::is_last(*num),
                None =>      false,
            }
        } else {
            // other upgrades don't have a limit.
            false
        }
    }
}

impl EventHandler for SandDropClicker {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
        // set up a fixed timestep
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
        }
        
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
            self.add_grain(x, y);
        }
        Ok(())
    }

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
    fn btn_txt(&self) -> &str {
        match self {
            Upgrade::BiggerContainer => "Buy Bigger Container",
            Upgrade::ParticleUpgrade => "Improve Sand Quality",
            Upgrade::MoreParticles =>   "Buy More Particles",
            Upgrade::AutoClicker =>     "Buy Auto Clicker",
        }
    }

    // Descripts the upgrade
    fn desc(&self) -> &str {
        match self {
            Upgrade::BiggerContainer => "This will increase your container size:",
            Upgrade::ParticleUpgrade => "This will allow you to drop better sand:",
            Upgrade::MoreParticles =>   "This will allow you to drop more sand per click:",
            Upgrade::AutoClicker =>     "This will drop sand for you:",
        }
    }

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

    fn is_last(num: u32) -> bool {
        if let Some(last) = SandParticle::iter().last() {
            num >= last as u32
        } else {
            false
        }
    }

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
}

#[derive(Debug)]
struct Grain {
    rect: Rect,
    rotation: f32,
    color: Color,
    x_v: f32,
    y_v: f32,
}

impl Grain {
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
            y_v: 50.0,
        }
    }

    fn center(&self) -> [f32;2] {
        [
            self.rect.x + self.rect.w / 2.0,
            self.rect.y + self.rect.h / 2.0
        ]
    }

    fn point(&self) -> [f32;2] {
        [
            self.rect.x,
            self.rect.y
        ]
    }

    fn size(&self) -> [f32;2] {
        [
            self.rect.w,
            self.rect.h
        ]
    }

    fn update(&mut self, dt: f32) {
        // move the grain downwards
        self.rect.translate([0.0, self.y_v * dt]);

        if self.rect.bottom() >= SCREEN_SIZE.1 {
            self.rect.y = SCREEN_SIZE.1 - self.rect.h;
            self.y_v = 0.0;
        }
    }

    fn draw_params(&self) -> DrawParam {
        DrawParam::default()
            .dest(self.center())
            .rotation(self.rotation)
            .scale(self.size())
            .offset([0.5, 0.5])
            .color(self.color)
    }
}
