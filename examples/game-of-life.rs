#![no_std]
#![no_main]

use gd32vf103xx_hal::delay::McycleDelay;
use panic_halt as _;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::style::PrimitiveStyle;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins};
use riscv_rt::entry;

const LCD_WIDTH: i32 = 160;
const LCD_HEIGHT: i32 = 80;

#[derive(Clone, Copy)]
struct Cell {
    // 00: dead -> dead
    // 01: alive -> dead
    // 10: dead -> alive
    // 11: alive -> alive
    state: u8,
}

impl Cell {
    pub fn alive(&self) -> bool {
        self.state & 1 > 0
    }

    // set next state as alive
    pub fn turn_alive(&mut self) {
        self.state |= 2
    }

    // switch to new state
    pub fn switch(&mut self) {
        self.state = self.state >> 1;
    }
}

const UNIVERSE_WIDTH: usize = 80;
const UNIVERSE_HEIGHT: usize = 40;

struct Universe {
    cells: [[Cell; UNIVERSE_HEIGHT]; UNIVERSE_WIDTH],
}

impl Universe {
    pub fn new() -> Self {
        let mut g = Universe {
            cells: [[Cell { state: 0 }; UNIVERSE_HEIGHT]; UNIVERSE_WIDTH],
        };

        // NOTE: set initial state here
        for x in 38..43 {
            g.cells[x][20].state = 1;
        }
        for y in 18..23 {
            g.cells[40][y].state = 1;
        }

        g
    }

    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count: usize = 0;

        for i in (UNIVERSE_WIDTH - 1 + x)..(x + 2 + UNIVERSE_WIDTH) {
            for j in (UNIVERSE_HEIGHT - 1 + y as usize)..(y as usize + 2 + UNIVERSE_HEIGHT) {
                let px = i % UNIVERSE_WIDTH;
                let py = j % UNIVERSE_HEIGHT;
                if px == x && py == y {
                    continue;
                }
                if self.cells[px as usize][py as usize].alive() {
                    count += 1;
                }
            }
        }

        count
    }

    // refresh universe
    pub fn update(&mut self) {
        for x in 0..UNIVERSE_WIDTH {
            for y in 0..UNIVERSE_HEIGHT {
                let neighbours = self.count_neighbors(x, y);
                if neighbours == 2 || neighbours == 3 {
                    // alive
                    self.cells[x as usize][y as usize].turn_alive();
                }
            }
        }
        // switch to new state
        for x in 0..UNIVERSE_WIDTH {
            for y in 0..UNIVERSE_HEIGHT {
                self.cells[x as usize][y as usize].switch();
            }
        }
    }

    fn draw_cell<T>(&self, display: &mut T, x: i32, y: i32) -> Result<(), T::Error>
    where
        T: DrawTarget<Rgb565>,
    {
        Rectangle::new(Point::new(x * 2, y * 2), Point::new(x * 2 + 1, y * 2 + 1))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN))
            .draw(display)?;
        Ok(())
    }

    // display
    pub fn show<T>(&self, display: &mut T) -> Result<(), T::Error>
    where
        T: DrawTarget<Rgb565>,
    {
        // Clear screen
        Rectangle::new(Point::new(0, 0), Point::new(LCD_WIDTH - 1, LCD_HEIGHT - 1))
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
            .draw(display)?;

        for x in 0..UNIVERSE_WIDTH {
            for y in 0..UNIVERSE_HEIGHT {
                if self.cells[x][y].alive() {
                    self.draw_cell(display, x as i32, y as i32)?;
                }
            }
        }

        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();
    let mut afio = dp.AFIO.constrain(&mut rcu);

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    let mut delay = McycleDelay::new(&rcu.clocks);

    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    // let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

    let mut u = Universe::new();

    loop {
        u.show(&mut lcd).unwrap();
        delay.delay_ms(500);
        u.update();
    }
}
