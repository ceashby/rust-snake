extern crate termion;

use rand::Rng;
use termion::event::{Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Write, stdout, Stdout};
use std::{time, thread};
use termion::async_stdin;

struct Player {
    points: Vec<Point>,
    direction: Direction,
}

impl Player {
    fn symbol(&self) -> String{
        match self.direction {
            Direction::Up => "↑".to_string(),
            Direction::Down => "↓".to_string(),
            Direction::Left => "←".to_string(),
            Direction::Right => "→".to_string(),
        }
    }

    fn head(&self) -> &Point {
        return self.points.last().unwrap();
    }
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq)]
struct Point {
    x: isize,
    y: isize,
}

impl Point {
    fn relative(&self, direction: &Direction) -> Point {
        match direction {
            Direction::Up => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

struct Board {
    width: usize,
    height: usize,
    player: Player,
    dots: Vec<Point>,
}

fn find_point(points: &Vec<Point>, x: isize, y: isize) -> Option<usize>{
    points.iter().position(|d| d.x == x && d.y == y)
}

impl Board {
    fn create_dot(&mut self){
        let mut rng = rand::thread_rng();
        loop{
            let x = rng.gen_range(0, self.width) as isize;
            let y = rng.gen_range(0, self.height) as isize;

            if find_point(&self.dots, x, y).is_none() &&
                find_point(&self.player.points, x, y).is_none(){
                self.dots.push(Point{x, y});
                return;
            }
        }
    }

    fn set_player_direction(&mut self, direction: Direction) {
        let len = self.player.points.len();
        if len > 1 && self.player.points[len - 2] == self.player.head().relative(&direction) {
            return;
        } else {
            self.player.direction = direction
        }
    }

    fn move_player(&mut self){
        let new_head = self.player.head().relative(&self.player.direction);

        self.player.points.push(new_head)
    }

    fn step(&mut self) {
        self.move_player();
        self.check_collisions()
    }

    fn check_collisions(&mut self){
        let result = find_point(
            &self.dots,
            self.player.head().x,
            self.player.head().y
        );

        match result {
            Some(index) => {
                self.dots.remove(index);
                self.create_dot()
            },
            None => {
                self.player.points.remove(0);
            },
        }

        if self.player.head().x as usize >= self.width ||
            self.player.head().y < 0 ||
            self.player.head().y as usize >= self.height ||
            self.player.head().x < 0 {
            panic!("Game Over");
        }

        let index = find_point(&self.player.points, self.player.head().x, self.player.head().y).unwrap();
        if index != self.player.points.len() - 1{
            panic!("Game Over");
        }
    }
}

fn draw(stdout: &mut RawTerminal<Stdout>, board: &Board){
//    write!(stdout,
//       "{}",
//       termion::clear::All
//    ).unwrap();

    for y in 0..board.height {
        let mut string = "".to_string();
        for x in 0..board.width {
            if y == board.player.head().y as usize && x == board.player.head().x as usize {
                string.push_str(&board.player.symbol());
            } else if find_point(&board.player.points, x as isize, y as isize).is_some(){
                string.push_str("•")
            }else if find_point(&board.dots, x as isize, y as isize).is_some() {
                string.push_str("·");
            }else{
                string.push_str(" ");
            }
        }
        string.push_str("█");
        writeln!(stdout, "{}{}", termion::cursor::Goto(1, y as u16 + 1), string).unwrap();
    }

    writeln!(stdout, "{}{}", termion::cursor::Goto(1, board.height as u16 + 1), "█".repeat(board.width + 1)).unwrap();
    writeln!(stdout, "{}", board.player.points.len() - 1).unwrap();

    stdout.flush().unwrap();
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().keys();

    let mut board = Board{
        width: 30,
        height: 20,
        dots: vec![],
        player: Player{
            direction: Direction::Right,
            points: vec![
                Point {
                    x: 0,
                    y: 0,
                }
            ],
        }
    };

    board.create_dot();
    board.create_dot();
    board.create_dot();
    board.create_dot();

    let mut iteration = 0;
    write!(stdout,
       "{}{}",
       termion::clear::All,
       termion::cursor::Hide,
    ).unwrap();

    loop {
        iteration +=1;
        thread::sleep(time::Duration::from_millis(10));
        let result = stdin.next();
        if let Some(Ok(key)) = result {
            match key {
                Key::Char('q') => break,
                Key::Left => board.set_player_direction(Direction::Left),
                Key::Right => board.set_player_direction(Direction::Right),
                Key::Up => board.set_player_direction(Direction::Up),
                Key::Down => board.set_player_direction(Direction::Down),
                _ => {},
            }
        }

        if iteration % 10 == 0 {
            board.step()
        }

        draw(&mut stdout, &board);
        stdout.flush().unwrap()
    }
}