use core::sync::atomic::{Ordering, AtomicU64};
use alloc::vec::Vec;
use terminal::{TerminalOutput, TerminalCharacter, Point, Resolution, STDOUT};
use drivers::{pit, ps2, vga};
use drivers::keyboard::{Ps2Keyboard, Keyboard, KeyEventType, keymap::codes::*};
use halt;

//const BLOCK: char = 219u8 as char;
const HEAD_CHAR: char = 2 as char;
lazy_static! {
    static ref RNG: Random = Random::new();
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Empty,
    Head,
    Body,
    Apple,
}

impl Cell {
    fn character(&self) -> TerminalCharacter {
        match self {
            Cell::Empty => TerminalCharacter::new(' ', color!(Black on Black)),
            Cell::Head => TerminalCharacter::new(HEAD_CHAR, color!(White on Black)),
            Cell::Body => TerminalCharacter::new(' ', color!(Green on Green)),
            Cell::Apple => TerminalCharacter::new(' ', color!(Red on Red))
        }
    }
}

struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    fn empty(width: usize, height: usize) -> Grid {
        Grid {
            cells: vec![Cell::Empty; width * height],
            width,
            height,
        }
    }

    fn set(&mut self, point: Point, cell: Cell) {
        self.cells[Grid::index(point)] = cell;
        STDOUT.write().set_char(cell.character(), point)
            .expect("failed to draw cell to screen");
    }

    fn get(&self, point: Point) -> &Cell {
        &self.cells[Grid::index(point)]
    }

    fn get_mut(&mut self, point: Point) -> &mut Cell {
        &mut self.cells[Grid::index(point)]
    }

    #[inline]
    fn index(point: Point) -> usize {
        if point.x >= 80 || point.y >= 25 {
            panic!("point out of bounds {} {}", point.x, point.y);
        }
        point.x + point.y * 80
    }
}

pub fn snake(controller: &mut ps2::Controller) {

    // Initialise keyboard
    let keyboard_device = controller.device(ps2::DevicePort::Keyboard);
    let mut keyboard = Ps2Keyboard::new(keyboard_device);
    if let Ok(_) = keyboard.enable() {
        info!("kbd: successfully enabled");
    } else {
        error!("kbd: enable unsuccessful");
        halt();
    }

    // Set up snake
    let mut ups = 20;
    let mut rng = Random::new();
    let mut snake = Snake::new();

    let res = STDOUT.read().resolution();
    let mut grid = Grid::empty(res.x as usize, res.y as usize);
    STDOUT.write().clear().expect("Error clearing screen");
    grid.set(generate_apple_pos(&grid), Cell::Apple);

    loop {
        pit::sleep(1000 / ups);

        // maybe extract this to fn or smth idk
        if let Ok(Some(event)) = keyboard.read_event() {
            if event.event_type != KeyEventType::Break {
                // Change direction of motion
                let new_direction = match event.keycode {
                    UP_ARROW | W => Some(Direction::Up),
                    DOWN_ARROW | S => Some(Direction::Down),
                    LEFT_ARROW | A => Some(Direction::Left),
                    RIGHT_ARROW | D => Some(Direction::Right),
                    _ => None,
                };
                if let Some(new_direction) = new_direction {
                    if new_direction != snake.direction.opposite() {
                        snake.direction = new_direction;
                    }
                }

                // Make the sleep time longer if snake is travelling up because the cells are higher
                // than they are wide
                match snake.direction {
                    Direction::Up | Direction::Down => ups = 10,
                    Direction::Left | Direction::Right => ups = 20,
                }

                // Revive snake if was dead
                if !snake.alive {
                    snake = Snake::new();
                    grid = Grid::empty(res.x as usize, res.y as usize);
                    STDOUT.write().clear().expect("Error clearing screen");
                    grid.set( generate_apple_pos(&grid), Cell::Apple);
                }
            }
        }

        snake.update(&mut grid);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn offset(&self, point: Point, amount: usize) -> Point {
        match self {
            Direction::Up => Point::new(point.x, point.y + amount),
            Direction::Down => Point::new(point.x, point.y - amount),
            Direction::Left => Point::new(point.x - amount, point.y),
            Direction::Right => Point::new(point.x + amount, point.y),
        }
    }

    fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

struct Snake {
    head: Point,
    direction: Direction,
    blocks: Vec<Point>,
    len: u16,
    first_tick: bool,
    alive: bool,
}

impl Snake {
    fn new() -> Snake {
        Snake {
            head: STDOUT.read().resolution().center(),
            direction: Direction::Right,
            blocks: Vec::with_capacity(128),
            len: 4,
            first_tick: true,
            alive: true,
        }
    }

    fn update(&mut self, grid: &mut Grid) {
        if !self.alive {
            return;
        }

        // Replace the previous head position with a snake body block
        if self.first_tick {
            self.first_tick = false;
        } else {
            grid.set(self.head, Cell::Body);
        }

        let move_result = self.try_move(grid);
        match move_result {
            MoveResult::Moved(point) => self.head = point,
            MoveResult::Win => {
                self.game_over(&format!("You win! Final score: {}", self.len - 4));
                return;
            }
            MoveResult::Lose => {
                self.game_over(&format!("You died! Final score: {}", self.len - 4));
                return;
            }
        }

        // Draw the head
        grid.set(self.head, Cell::Head);

        // Update the train
        if self.blocks.len() < self.len as usize {
            self.blocks.insert(0, self.head);
        } else {
            // Clear the tail point
            if let Some(tail) = self.blocks.last() {
                grid.set(tail.clone(), Cell::Empty);
            }

            self.blocks.rotate_right(1);
            self.blocks[0] = self.head;
        }
    }

    fn try_move(&mut self, grid: &mut Grid) -> MoveResult {
        let moved_head = self.direction.offset(self.head, 1);
        if !STDOUT.read().in_bounds(moved_head) {
            return MoveResult::Lose;
        }

        if self.len as usize == grid.width * grid.height {
            return MoveResult::Win;
        }

        let cell = grid.get(moved_head);
        match cell {
            Cell::Body => return MoveResult::Lose,
            Cell::Apple => {
                self.len += 1;
                grid.set(generate_apple_pos(&grid), Cell::Apple);
            }
            _ => (),
        }

        MoveResult::Moved(moved_head)
    }

    fn game_over(&mut self, message: &str) {
        self.alive = false;

        let mut stdout = STDOUT.write();

        let cursor = stdout.resolution().center();
        let cursor = Point::new(cursor.x - message.len() / 2, cursor.y);

        stdout.set_color(color!(White on Black)).expect("Error setting color!");
        stdout.set_cursor_pos(cursor).expect("Error setting cursor pos!");
        stdout.write_string(message).expect("Error writing string!");
    }
}

enum MoveResult {
    Moved(Point),
    Win,
    Lose,
}

fn generate_apple_pos(grid: &Grid) -> Point {
    loop {
        let x = RNG.next_bounded(grid.width as u64) as usize;
        let y = RNG.next_bounded(grid.height as u64) as usize;
        let point = Point::new(x, y);
        if *grid.get(point) == Cell::Empty {
            return point;
        }
    }
}

struct Random {
    seed: AtomicU64,
}

impl Random {
    fn new() -> Random {
        let time = pit::time_ms() as u64;
        Random {
            seed: AtomicU64::new(time ^ 2246577883182828989),
        }
    }

    fn with_seed(seed: u64) -> Random {
        Random { seed: AtomicU64::new(seed) }
    }

    /// Thanks to https://stackoverflow.com/a/3062783/4871468 and
    /// https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use (glibc's
    /// values used here).
    fn next_bounded(&self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn next(&self) -> u64 {
        const A: u64 = 1103515245;
        const M: u64 = 1 << 31;
        const C: u64 = 12345;

        loop {
            let seed = self.seed.load(Ordering::SeqCst); // TODO ordering
            let next = (A * seed + C) % M;
            if self.seed.compare_and_swap(seed, next, Ordering::SeqCst) == seed {
                return next;
            }
        }
    }
}
