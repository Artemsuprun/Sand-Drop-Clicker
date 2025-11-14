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
    time::{Duration, Instant}
};
use ggegui::{egui, Gui};
use ggez::{
	ContextBuilder, Context, GameResult, glam, timer,
	event::{ self, EventHandler}, 
	graphics::{ self, DrawParam, Color }
};

fn main() {
	let (mut ctx, event_loop) = ContextBuilder::new("SandDropClicker", "Artem Suprun")
        .window_setup(ggez::conf::WindowSetup::default().title("Sand Drop Clicker"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0))
        .build()
        .unwrap();
	let state = GameState::new(&mut ctx);
	event::run(ctx, event_loop, state);
}

struct GameState {
    game: SandDropClicker,
	gui: Gui,
}

impl GameState {
	pub fn new(ctx: &mut Context) -> Self {
		Self {
            game: SandDropClicker::new(),
			gui: Gui::new(ctx),
		}
	}

    // loads the options window
    fn options(&mut self, ctx: &mut Context) {
        let gui_ctx = self.gui.ctx();
        egui::Window::new("Sand Drop Clicker").show(&gui_ctx, |ui| {
			ui.label("Click the button to earn money!");
			if ui.button("Convert").clicked() {
				self.game.make_money();
			}
		});
    }
}

impl EventHandler for GameState {
	fn update(&mut self, ctx: &mut Context) -> GameResult {
		
        self.options(ctx);
		self.gui.update(ctx);
		Ok(())
	}

    // draw the game state
	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
		canvas.draw(&self.gui, DrawParam::default());
        self.game.draw(&mut canvas);
		canvas.finish(ctx)
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
        // Ignore clicks if the pointer is over the GUI
        if self.gui.ctx().wants_pointer_input() {
            return Ok(());
        }
        self.game.drop_sand();
        Ok(())
    }
}

struct SandDropClicker {
    money: i64,
    particles: HashMap<SandParticle, Vec<Grain>>,
    upgrades: HashMap<Upgrade, u32>,
    total_clicks: u32,
    total_time: Duration,
}

impl SandDropClicker {
    fn new() -> Self {
        SandDropClicker {
            money: 0,
            particles: HashMap::new(),
            upgrades: HashMap::new(),
            total_clicks: 0,
            total_time: Duration::new(0, 0),
        }
    }

    fn draw(&self, canvas: &mut graphics::Canvas) {
        // Drawing logic here
    }

    // simulates dropping a sand particle
    fn drop_sand(&mut self) {
        println!("Dropping sand particle!");
        /*
        self.particles
            .entry(SandParticle::Sand)
            .and_modify(|e| *e += 1)
            .or_insert(1);
        */
    }

    // converts sand particles to money
    fn make_money(&mut self) {
        println!("Making money!");
    }
}

// Game structures and logic

enum Upgrade {
    BiggerContainer,
    MoreParticles,
    AutoClicker,
}

#[derive(Hash, Eq, PartialEq, Debug)]
enum SandParticle {
    Water,
    Sand,
    Coal,
    Gold,
    Diamond ,
}

struct Grain {
    x: f32,
    y: f32,
    particle_type: SandParticle,
}