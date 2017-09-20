use sdl2;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::render::Canvas;

const BG_COLOR: Color = Color { r: 53, g: 59, b: 115, a: 0xFF };
const FG_COLOR: Color = Color { r: 255, g: 255, b: 41, a: 0xFF };

pub struct Display {
    working_screen: [[bool; 32]; 64],
    canvas: Canvas<Window>
}

impl Display {
    pub fn new(sdl_context: &sdl2::Sdl) -> Display {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Alvin", 640, 320)
            .position_centered().build().unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        canvas.set_scale(10.0, 10.0);
        canvas.set_draw_color(BG_COLOR);
        canvas.clear();

        Display {
            working_screen: [[false; 32]; 64],
            canvas
        }
    }

    pub fn screen_dimensions(&self) -> (usize, usize) {
        (self.working_screen.len(), self.working_screen[0].len())
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(BG_COLOR);
        self.working_screen = [[false; 32]; 64];
        self.canvas.clear();
        self.canvas.present();
    }

    pub fn draw(&mut self, x: u8, y: u8, mut sprite: u8) -> bool {
        let mut collision = false;

        let x = x as usize;
        let mut y = y as usize;

        for _ in 0x0..0x8 {
            let highest_bit = (sprite & 0x80) == 0x80;
            sprite = sprite << 1;

            collision |= self.working_screen[y][x];

            self.working_screen[y][x] ^= highest_bit;

            if y == self.working_screen.len() - 1 {
                y = 0;
            } else {
                y += 1;
            }
        }

        return collision;
    }

    pub fn render(&mut self) {
        for i in 0..self.working_screen.len() {
            for j in 0..self.working_screen[i].len() {
                let color = if self.working_screen[i][j] {
                    FG_COLOR
                } else {
                    BG_COLOR
                };

                self.canvas.set_draw_color(color);
                self.canvas.draw_point(Point::new(i as i32, j as i32));
            }
        }

        self.canvas.present();
    }
}