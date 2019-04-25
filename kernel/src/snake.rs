use core::sync::atomic::{Ordering, AtomicU64};
use alloc::vec::Vec;
use crate::terminal::{TerminalOutput, TerminalCharacter, Point, STDOUT};
use crate::drivers::{pit, ps2};
use crate::drivers::keyboard::{Ps2Keyboard, Keyboard, KeyEventType};
use crate::halt;

const HEAD_CHAR: char = 2 as char;
const BASE_LENGTH: u16 = 4;

lazy_static! {
    static ref RNG: Random = Random::new();
}

struct Game<'a> {
    grid: Grid,
    snake: Snake,
    ups: usize,
    keyboard: &'a mut Ps2Keyboard<'a>,
    highscore: u16,
}

impl<'a> Game<'a> {
    fn new(keyboard: &'a mut Ps2Keyboard<'a>) -> Game<'a> {
        let res = STDOUT.read().resolution().expect("Terminal must have resolution");

        Game {
            grid: Grid::empty(res.x as usize, res.y as usize),
            snake: Snake::new(),
            ups: 20,
            keyboard,
            highscore: 0,
        }
    }

    fn run(&mut self) {
        STDOUT.write().clear().expect("Error clearing screen");
        self.notification("Welcome to snake!");
        self.initialize();

        loop {
            pit::sleep(1000 / self.ups);

            let new_direction = self.get_input();

            if let Some(new_direction) = new_direction {
                if new_direction != self.snake.direction.opposite() {
                    self.snake.direction = new_direction;
                }
            }

            // Make the UPS lower (to lower apparent velocity) snake is travelling up because
            // the cells are higher than they are wide
            self.ups = match self.snake.direction {
                Direction::Up | Direction::Down => 10,
                Direction::Left | Direction::Right => 20,
            };

            // See if the game is over and if so restart it
            let win = match self.snake.update(&mut self.grid) {
                MoveResult::Win => true,
                MoveResult::Lose => false,
                _ => continue,
            };

            self.restart(win);
        }
    }

    fn get_input(&mut self) -> Option<Direction> {
        use crate::drivers::keyboard::keymap::codes::*;

        let event = self.keyboard.read_event().expect("Error reading keyboard input!")?;

        if event.event_type != KeyEventType::Break {
            match event.keycode {
                UP_ARROW | W => Some(Direction::Up),
                DOWN_ARROW | S => Some(Direction::Down),
                LEFT_ARROW | A => Some(Direction::Left),
                RIGHT_ARROW | D => Some(Direction::Right),
                _ => None,
            }
        } else {
            None
        }
    }

    fn initialize(&mut self) {
        self.snake = Snake::new();
        self.grid.clear();
        STDOUT.write().clear().expect("Error clearing screen");
        self.grid.set(generate_apple_pos(&self.grid), Cell::Apple);
    }

    fn restart(&mut self, win: bool) {
        let won = if win { "win" } else { "lose" };
        let highscore = if self.snake.score() > self.highscore {
            self.highscore = self.snake.score();
            " New highscore!"
        } else {
            ""
        };
        self.notification(
            &format!("You {}! Final score: {}.{}", won, self.snake.score(), highscore)
        );

        self.initialize();
    }

    fn notification(&mut self, message: &str) {
        let old_color = STDOUT.read().color().expect("Terminal must support colors");
        STDOUT.write().set_color(color!(White on Black)).expect("Error setting color!");
        let center = STDOUT.read().resolution().expect("Terminal must have resolution").center();

        centered_text(message, center.x, center.y);
        centered_text("Press any key to continue...", center.x, center.y - 1);

        STDOUT.write().set_color(old_color).expect("Error setting color!");

        pit::sleep(1000);

        loop {
            if let Ok(Some(event)) = self.keyboard.read_event() {
                if event.event_type == KeyEventType::Break {
                    break;
                }
            }
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

    fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = Cell::Empty;
        }
    }

    fn set(&mut self, point: Point, cell: Cell) {
        let index = self.index(point);
        self.cells[index] = cell;
        STDOUT.write().set_char(cell.character(), point)
            .expect("failed to draw cell to screen");
    }

    fn get(&self, point: Point) -> &Cell {
        &self.cells[self.index(point)]
    }
    
    #[inline]
    fn contains(&self, point: Point) -> bool {
        point.x < self.width && point.y < self.height
    }

    #[inline]
    fn index(&self, point: Point) -> usize {
        if !self.contains(point) {
            panic!("point out of bounds {} {}", point.x, point.y);
        }
        point.x + point.y * self.width
    }
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Snake {
    head: Point,
    direction: Direction,
    blocks: Vec<Point>,
    len: u16,
}

impl Snake {
    fn new() -> Snake {
        Snake {
            head: STDOUT.read().resolution().expect("Terminal must have resolution").center(),
            direction: Direction::Right,
            blocks: Vec::with_capacity(128),
            len: BASE_LENGTH,
        }
    }

    fn update(&mut self, grid: &mut Grid) -> MoveResult {
        // Replace the previous head position with a snake body block
        if !self.blocks.is_empty() {
            grid.set(self.head, Cell::Body);
        }

        let move_result = self.try_move(grid);
        match move_result {
            MoveResult::Moved(point) => self.head = point,
            MoveResult::Win => return MoveResult::Win,
            MoveResult::Lose => return MoveResult::Lose,
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

        move_result
    }

    fn try_move(&mut self, grid: &mut Grid) -> MoveResult {

        let moved_head = match self.direction.offset(self.head, 1) {
            Some(head) if grid.contains(head) => head,
            _ => return MoveResult::Lose,
        };

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

    fn score(&self) -> u16 {
        self.len - BASE_LENGTH
    }
}

impl Direction {
    /// Returns none if it would overflow
    fn offset(&self, point: Point, amount: usize) -> Option<Point> {
        match self {
            Direction::Up => Some(Point::new(point.x, point.y + amount)),
            Direction::Down => if point.y >= amount {
                Some(Point::new(point.x, point.y - amount))
            } else {
                None
            },
            Direction::Left => if point.x >= amount {
                Some(Point::new(point.x - amount, point.y))
            } else {
                None
            },
            Direction::Right => Some(Point::new(point.x + amount, point.y)),
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

enum MoveResult {
    Moved(Point),
    Win,
    Lose,
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

    /// Thanks to https://stackoverflow.com/a/3062783/4871468 and
    /// https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use
    /// (glibc's values used here).
    fn next_bounded(&self, bound: u64) -> u64 {
        self.next() % bound
    }

    fn next(&self) -> u64 {
        const A: u64 = 1103515245;
        const M: u64 = 1 << 31;
        const C: u64 = 12345;

        let mut seed = self.seed.load(Ordering::SeqCst);
        loop {
            let next = (A.wrapping_mul(seed) + C) % M;
            let cas_result = self.seed.compare_and_swap(seed, next, Ordering::SeqCst);

            if cas_result == seed {
                return next;
            } else {
                seed = cas_result;
            }
        }
    }
}


pub fn snake(controller: &mut ps2::Controller) {
    let keyboard_device = controller.device(ps2::DevicePort::Keyboard);
    let mut keyboard = Ps2Keyboard::new(keyboard_device);
    if let Ok(_) = keyboard.enable() {
        info!("kbd: successfully enabled");
    } else {
        error!("kbd: enable unsuccessful");
        halt();
    }

    let mut game = Game::new(&mut keyboard);
    game.run()
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

fn centered_text(message: &str, x_center: usize, y: usize) {
    let mut stdout = STDOUT.write();

    let cursor = Point::new(x_center - message.len() / 2, y);
    let old_cursor = stdout.cursor_pos().expect("Terminal must support cursor");

    stdout.set_cursor_pos(cursor).expect("Error setting cursor pos!");
    stdout.write_string(message).expect("Error writing string!");
    stdout.set_cursor_pos(old_cursor).expect("Error setting cursor pos!");
}
