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
	ContextBuilder, Context, GameResult, glam, timer,
	event::{ self, EventHandler }, 
	graphics::{ self, DrawParam, Color, Text },
    input::keyboard::{KeyCode, KeyMods, KeyInput}
};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;

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
    fn options_gui(&mut self, ctx: &mut Context) {
        let gui_ctx = self.gui.ctx();
        egui::Window::new("Sand Drop Clicker").show(&gui_ctx, |ui| {
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
                let cost = self.game.cost(upgrade);
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

    fn draw_game_info(&self, canvas: &mut graphics::Canvas) {
        if self.show_info {  
            let total_time = self.game.total_time.as_secs();
            let total_clicks = self.game.total_clicks;
            let txt = Text::new(
                format!("Total Time: {} seconds \nTotal Clicks: {}", 
                    total_time, 
                    total_clicks)
            );
            canvas.draw(
                &txt,
                graphics::DrawParam::from([10.0, 10.0])
                    .color(Color::WHITE),
            );
        }
    }

    fn draw_player_info(&self, canvas: &mut graphics::Canvas) {

    }
}

impl EventHandler for GameState {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
        // set up a fixed timestep
        let fps: u32 = 30;
        while ctx.time.check_update_time(fps) {
            let seconds = 1.0 / fps as f32;
            self.game.update(seconds);
        }
        self.options_gui(ctx);
		self.gui.update(ctx);
		Ok(())
	}

    // draw the game state
	fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // clear the screen
		let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
		
        // draw the GUI and game
        canvas.draw(&self.gui, DrawParam::default());
        self.game.draw(&mut canvas);

        // draw game info if enabled
        self.draw_game_info(&mut canvas);
        self.draw_player_info(&mut canvas);

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

    fn draw(&self, canvas: &mut graphics::Canvas) {
        // draw the grain particle

    }

    fn update(&mut self, seconds: f32) {
        // update the total_time stat
        self.total_time += Duration::from_secs_f32(seconds);

        // update the position of the falling particles.
        for (_, grains) in self.particles.iter() {
            
        }
    }

    // simulates dropping a sand particle
    fn add_grain(&mut self, x: f32, y: f32) {
        // increment total clicks
        self.total_clicks += 1;

        // add a sand particle at (x, y)
        let grain = Grain {
            x,
            y,
            particle_type: SandParticle::Sand,
        };
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

    fn cost(&self, upgrade: Upgrade) -> i64 {
        100 // Placeholder cost logic
        // base_cost * 1.1^M
    }

    fn is_full(&self) -> bool {
        // base container size
        let base_size = 25;
        let mut container = base_size;
        
        // count the number of particles in the container
        let mut amount = 0;
        for (_, grains) in self.particles.iter() {
            amount += grains.len();
        }

        container > amount
    }
}

#[derive(Hash, Eq, PartialEq, Debug, EnumIter, Clone, Copy)]
enum Upgrade {
    BiggerContainer,
    MoreParticles,
    AutoClicker,
}

impl Upgrade {
    fn unlock_cost(&self) -> f64 {
        match self {
            Upgrade::BiggerContainer => 50.0,
            Upgrade::MoreParticles => 100.0,
            Upgrade::AutoClicker => 250.0,
        }
    }

    fn button_text(&self) -> &'static str {
        match self {
            Upgrade::BiggerContainer => "Buy Bigger Container",
            Upgrade::MoreParticles => "Buy More Particles",
            Upgrade::AutoClicker => "Buy Auto Clicker",
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
enum SandParticle {
    Water,
    Sand,
    Coal,
    Gold,
    Diamond ,
}

impl SandParticle {
    fn value(&self) -> f64 {
        match self {
            SandParticle::Water => 0.01,
            SandParticle::Sand => 0.05,
            SandParticle::Coal => 0.10,
            SandParticle::Gold => 0.50,
            SandParticle::Diamond => 1.00,
        }
    }
}

#[derive(Debug)]
struct Grain {
    x: f32,
    y: f32,
    particle_type: SandParticle,
}