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
use ggegui::{egui, Gui};
use ggez::{
	ContextBuilder, Context, GameResult, glam,
	event::{ self, EventHandler }, 
	graphics::{ self, DrawParam, Color, Text },
    input::keyboard::{KeyCode, KeyMods, KeyInput}
};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

// Global Variable
const FPS: u32 = 30;

// Set up and run the game
fn main() {
	let (mut ctx, event_loop) = ContextBuilder::new("SandDropClicker", "Artem Suprun")
        .window_setup(ggez::conf::WindowSetup::default().title("Sand Drop Clicker"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0))
        .build()
        .unwrap();
	let state = GameState::new(&mut ctx);
	event::run(ctx, event_loop, state);
}

// Main game state
// holds the game logic and GUI
struct GameState {
    game: SandDropClicker,
	gui: Gui,
    unlock: HashSet<Upgrade>,
    show_info: bool,
}

impl GameState {
	pub fn new(ctx: &mut Context) -> Self {
		Self {
            game: SandDropClicker::new(),
			gui: Gui::new(ctx),
            unlock: HashSet::new(),
            show_info: false,
		}
	}

    // loads the options window
    fn options_gui(&mut self) {
        let gui_ctx = self.gui.ctx();
        egui::Window::new("Options").show(&gui_ctx, |ui| {
            // Display instructions
			ui.label("Click the button to earn money!");
			if ui.button("Convert").clicked() {
				self.game.make_money();
			}
            // display money
            ui.label(format!("Money: {}", self.game.money));

            // show available upgrades
            ui.separator();
            if self.unlock.is_empty() {
                ui.label("No upgrades available yet. Keep clicking!");
            } else {
                ui.label("Available Upgrades:");
            }
            for upgrade in Upgrade::iter() {
                let goal = upgrade.unlock_cost();
                let cost = self.game.upgrade_cost(upgrade);
                if self.unlock.contains(&upgrade) {
                    if ui.button(format!("{}", upgrade.button_text())).clicked() {
                        // fix sanddropclicker.cost and add logic here
                    }
                } else if self.game.money >= goal {
                    self.unlock.insert(upgrade);
                }
            }
		});
    }
}

impl EventHandler for GameState {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
        // set up a fixed timestep
        while ctx.time.check_update_time(FPS) {
            let seconds = 1.0 / FPS as f32;
            self.game.update(seconds);
        }
        self.options_gui();
		self.gui.update(ctx);
		Ok(())
	}

    // draw the game state
	fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // clear the screen
		let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
		
        // draw the GUI and game
        self.game.draw(ctx, &mut canvas);
        canvas.draw(&self.gui, DrawParam::default());

        // draw game info
        if self.show_info {
            self.game.player_info(&mut canvas);
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
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), ggez::GameError> {
        // Ignore clicks if the pointer is over the GUI or the container is full
        if !self.gui.ctx().wants_pointer_input() && self.game.is_full() {
            self.game.add_grain(x, y);
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

struct SandDropClicker {
    money: f64,
    particles: HashMap<SandParticle, Vec<Grain>>,
    upgrades: HashMap<Upgrade, u32>,
    total_clicks: u32,
    total_time: std::time::Duration,
}

impl SandDropClicker {
    fn new() -> Self {
        SandDropClicker {
            money: 0.0,
            particles: HashMap::new(),
            upgrades: HashMap::new(),
            total_clicks: 0,
            total_time: Duration::new(0, 0),
        }
    }

    fn draw(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) {
        // draw the grain particle
        for (_, grains) in self.particles.iter() {
            for grain in grains.iter() {
                grain.draw(canvas);
            }
        }

        // draw the player stat
        self.game_info(canvas);
    }

    fn update(&mut self, seconds: f32) {
        // update the total_time stat
        self.total_time += Duration::from_secs_f32(seconds);

        // update the position of the falling particles.
        for (_, grains) in self.particles.iter_mut() {
            for grain in grains.iter_mut() {
                grain.logic();
            }
        }
    }

    // simulates dropping a sand particle
    fn add_grain(&mut self, x: f32, y: f32) {
        // increment total clicks
        self.total_clicks += 1;

        // add a sand particle at (x, y)
        let sand = self.rand_sand();
        let size = 5.0;
        let grain = Grain {
            x: x,
            y: y,
            size: size,
            a: 0.0,
            particle: sand,
        };
        // Add the grain to the specific particle location.
        self.particles
            .entry(SandParticle::Sand)
            .or_insert_with(Vec::new)
            .push(grain);
    }

    // converts sand particles to money
    fn make_money(&mut self) {
        // sell all sand particles for money
        let mut earned = 0.0;
        for (particle, grains) in self.particles.iter_mut() {
            let count = grains.len() as f64;
            let value = particle.value();
            earned += count * value;
            grains.clear();
        }
        self.money += earned;
    }

    fn upgrade_cost(&self, upgrade: Upgrade) -> i64 {
        100 // Placeholder cost logic
        // base_cost * 1.1^M
    }

    fn is_full(&self) -> bool {
        // container size
        let size = self.get_size();
        let amount = self.get_amount();
        size > amount
    }

    fn get_size(&self) -> u32{
        // base container size
        let base_size = 25;
        let mut container_size = base_size;
        // some logic later

        container_size
    }

    fn get_amount(&self) -> u32 {
        // count the amount of particles in the container
        let mut amount = 0;
        for (_, grains) in self.particles.iter() {
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

    fn rand_sand(&self) -> SandParticle {
        SandParticle::Sand
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
    fn unlock_cost(&self) -> f64 {
        match self {
            Upgrade::BiggerContainer => 50.0,
            Upgrade::ParticleUpgrade => 75.0,
            Upgrade::MoreParticles =>   100.0,
            Upgrade::AutoClicker =>     250.0,
        }
    }

    fn button_text(&self) -> &'static str {
        match self {
            Upgrade::BiggerContainer => "Buy Bigger Container",
            Upgrade::ParticleUpgrade => "Improve Sand Quality",
            Upgrade::MoreParticles =>   "Buy More Particles",
            Upgrade::AutoClicker =>     "Buy Auto Clicker",
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
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
    fn value(&self) -> f64 {
        match self {
            SandParticle::Sand =>       1.0,
            SandParticle::Quartz =>     1.0,
            SandParticle::Shell =>      1.0,
            SandParticle::Coral =>      1.0,
            SandParticle::Pinksand =>   1.0,
            SandParticle::Volcanic =>   1.0,
            SandParticle::Glauconite => 1.0,
            SandParticle::Gemstones =>  1.0,
            SandParticle::Iron =>       1.0,
            SandParticle::Starsand =>   1.0,
            SandParticle::Gold =>       1.0,
            SandParticle::Diamond =>    1.0,
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
}

#[derive(Debug)]
struct Grain {
    //rect: graphics::Rect,
    x: f32,
    y: f32,
    size: f32,
    a: f32,
    particle: SandParticle,
}

impl Grain {
    fn point(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    fn size(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.size,
            y: self.size,
        }
    }

    fn logic(&mut self) {
        self.y += 3.0;
    }

    fn draw(&self, canvas: &mut graphics::Canvas) {
        let color = self.particle.color();
        canvas.draw(
            &graphics::Quad, 
            graphics::DrawParam::new()
                .dest(self.point())
                .rotation(self.a)
                .offset(glam::Vec2::new(0.5, 0.5))
                .scale(self.size())
                .color(color),
        );
    }
}
