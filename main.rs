extern crate sdl2;

const GRID_X_SIZE : u32 = 40;
const GRID_Y_SIZE : u32 = 30;
const DOT_SIZE_IN_PXS : u32 = 20;
const MARGIN : i32 = 1;

use std::ops::Add;
use std::path::*;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use std::time::Duration;
use rand::Rng;


pub enum GameState { Playing, Paused, Over}
pub enum PlayerDirection { Up,Down,Right,Left}

#[derive(Clone, Copy)]
pub struct Point(pub i32,pub i32);

pub struct GameContext{
    pub player_position : Vec<Point>,
    pub player_direction : PlayerDirection,
    pub food : Point,
    pub state : GameState,
    pub score : u32
}

impl Add<Point> for Point{
    type Output = Point;
    fn add(self, rhs:Point) -> Self::Output{
        Point(self.0 + rhs.0,self.1 + rhs.1)
    }

}

impl Default for GameContext{
    fn default() -> Self {
        Self::new()
    }
}

impl GameContext {
    pub fn new() -> GameContext{
        GameContext { 
            player_position:vec![Point(3,1),Point(2,1),Point(1,1)], 
            player_direction: PlayerDirection::Right, 
            food: Point(3,3), 
            state: GameState::Playing,
            score: 0
        }
    }

    pub fn next_tick(&mut self){
        if let GameState::Paused = self.state {return;}
        if let GameState::Over = self.state {return;}

        let head_position = self.player_position.first().unwrap();
        let next_head_position = match self.player_direction {
            PlayerDirection::Up => *head_position + Point(0,-1),
            PlayerDirection::Down => *head_position + Point(0,1),
            PlayerDirection::Left => *head_position + Point(-1,0),
            PlayerDirection::Right => *head_position + Point(1,0),
        };

        //game over check
        if next_head_position.0 < 0 || next_head_position.0 >= GRID_X_SIZE.try_into().unwrap() || 
        next_head_position.1 < 0 || next_head_position.1 >= GRID_Y_SIZE.try_into().unwrap() {
            self.state = GameState::Over;
        }

        //check if apple eaten
        if next_head_position.0 == self.food.0 && next_head_position.1 == self.food.1 {
            self.score+=1;
            self.respawn_food();
        }
        else {
            self.player_position.pop();
        }

        //check for no self intersections
        for u in &self.player_position {
            if next_head_position.0 == u.0 && next_head_position.1 == u.1{
                self.state = GameState::Over;
                return;
            }
        }

        //if ok continue
        self.player_position.reverse();
        self.player_position.push(next_head_position);
        self.player_position.reverse();
    }

    fn respawn_food(&mut self){
        //pick random unoccupied position
        let mut random = rand::thread_rng();
        let mut pos : u32;
        
        'generate: loop{
            pos = random.gen_range(0..GRID_X_SIZE*GRID_Y_SIZE);
        
            for x in &self.player_position{
                if x.0 as u32 == pos%GRID_X_SIZE && x.1 as u32 == pos/GRID_X_SIZE{
                    continue 'generate;
                }
            }
            
            break 'generate;
        }   

        self.food.0 = (pos%GRID_X_SIZE) as i32;
        self.food.1 = (pos/GRID_X_SIZE) as i32;
    }

    pub fn move_up(&mut self){
        if let PlayerDirection::Down = self.player_direction {return}
            self.player_direction = PlayerDirection::Up;
    }
    pub fn move_down(&mut self){
        if let PlayerDirection::Up = self.player_direction {return}
        self.player_direction = PlayerDirection::Down;
    }
    pub fn move_right(&mut self){
        if let PlayerDirection::Left = self.player_direction {return}
        self.player_direction = PlayerDirection::Right;
    }
    pub fn move_left(&mut self){
        if let PlayerDirection::Right = self.player_direction {return}
        self.player_direction = PlayerDirection::Left;
    }
    pub fn toggle_pause(&mut self){
        self.state = match self.state{
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            GameState::Over => GameState::Over
        };
    }
    pub fn new_game(&mut self){
        if let GameState::Over = self.state {
            *self = GameContext::new(); 
        }
    }
}

pub struct Renderer {
    canvas : WindowCanvas
}

impl Renderer {
    pub fn new(window : Window) -> Result<Renderer,String>{
        let canvas = window.into_canvas().build()
        .map_err(|e| e.to_string())?;
        Ok(Renderer{canvas})
    }

    fn draw_dot(&mut self,point : &Point) -> Result<(),String>{
        let Point(x,y) = point;
        self.canvas.fill_rect(Rect::new(
            x*DOT_SIZE_IN_PXS as i32 + MARGIN,
            y*DOT_SIZE_IN_PXS as i32 + MARGIN,
            DOT_SIZE_IN_PXS - (2*MARGIN as u32),
            DOT_SIZE_IN_PXS - (2*MARGIN as u32),
        ))?;
        Ok(())
    }

    pub fn draw(&mut self, context : &GameContext) -> Result<(), String> {
        self.draw_background(context);
        self.draw_player(context)?;
        self.draw_food(context)?;
        self.draw_text(context)?;
        self.canvas.present();

        Ok(())
    }

    fn draw_background(&mut self,context : &GameContext){
        let color = match context.state{
            GameState::Playing => Color::RGB(0,0,0),
            GameState::Paused => Color::RGB(30,30,30),
            GameState::Over => Color::RGB(0,0,0)
        };
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    fn draw_player(&mut self, context : &GameContext) -> Result<(),String> {
        if let GameState::Over = context.state {return Ok(());}
        self.canvas.set_draw_color(Color::GREEN);
        for point in &context.player_position {
            self.draw_dot(point)?;
        }
        Ok(())
    }

    fn draw_food(&mut self,context : &GameContext) -> Result<(),String>{
        if let GameState::Over = context.state {return Ok(());}
        self.canvas.set_draw_color(Color::RED);
        self.draw_dot(&context.food)?;
        Ok(())
    }

    fn draw_text(&mut self, context : &GameContext) -> Result<(),String>{
        if let GameState::Playing = context.state {return Ok(());}
        
        let text_creator = self.canvas.texture_creator();
        let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
        let font_path : &Path = Path::new("fonts/heavy_heap.otf");
        let font = ttf_context.load_font(font_path,128)?;
        
      //  font.set_style(sdl2::ttf::FontStyle::BOLD);

        let mut msg = "GAME OVER".to_string();
        let mut col = Color::RGB(255,20,147);
        if let GameState::Paused = context.state {
            msg = "PAUSED".to_string();
            col = Color::RGB(0,255,255);
        }

        let surface = font
        .render(&msg)
        .blended(col)
        .map_err(|e| e.to_string())?;
        
        let texture = text_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

        let target = Rect::new(100, 150, 600, 300);
        self.canvas.copy(&texture, None, Some(target))?;

        Ok(())
    }
}

pub fn main() -> Result<(),String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
    .window("Rnake", GRID_X_SIZE * DOT_SIZE_IN_PXS, GRID_Y_SIZE * DOT_SIZE_IN_PXS)
    .position_centered()
    .opengl()
    .build()
    .map_err(|e| e.to_string())?;


    let mut event_pump = sdl_context.event_pump()?;


    let mut context = GameContext::new();
    let mut renderer = Renderer::new(window)?;

    let mut tick_counter = 0;

    'running: loop {
        for event in event_pump.poll_iter(){
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode : Some(keycode), .. 
                } => {
                    match keycode {
                        Keycode::W => context.move_up(),
                        Keycode::Up => context.move_up(),
                        Keycode::A => context.move_left(),
                        Keycode::Left => context.move_left(),
                        Keycode::D => context.move_right(),
                        Keycode::Right => context.move_right(),
                        Keycode::S => context.move_down(),
                        Keycode::Down => context.move_down(),
                        Keycode::R => context.new_game(),
                        Keycode::P => context.toggle_pause(),
                        Keycode::Escape => break 'running,
                        _ => {}
                    }
                }
                 _ => {}
            }
        }

        tick_counter+=1;
        if tick_counter %3 == 0{
            context.next_tick();
            tick_counter = 0;
        }
        
        renderer.draw(&context)?;
        ::std::thread::sleep(Duration::new(0,1_000_000_000u32/30))
    }

    Ok(())
}