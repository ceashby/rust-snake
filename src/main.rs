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
    players: Vec<Player>,
    dots: Vec<Point>,
}

fn find_point(points: &Vec<Point>, x: isize, y: isize) -> Option<usize>{
    points.iter().position(|d| d.x == x && d.y == y)
}

impl Board {
    fn create_dot(&self) -> Point{
        let mut rng = rand::thread_rng();
        loop{
            let x = rng.gen_range(0, self.width) as isize;
            let y = rng.gen_range(0, self.height) as isize;

            if find_point(&self.dots, x, y).is_none() &&
                find_point(&self.players[0].points, x, y).is_none() &&
                find_point(&self.players[1].points, x, y).is_none(){
                return Point{x, y};
            }
        }
    }

    fn set_player_direction(&mut self, direction: Direction, player_index: usize) {
        let len = self.players[player_index].points.len();
        if len > 1 && self.players[player_index].points[len - 2] == self.players[player_index].head().relative(&direction) {
            return;
        } else {
            self.players[player_index].direction = direction
        }
    }

    fn move_player(&mut self, player_index: usize){
        let new_head = self.players[player_index].head().relative(&self.players[player_index].direction);

        self.players[player_index].points.push(new_head)
    }

    fn step(&mut self) {
        self.move_player(1);
        self.move_player(2);
        self.check_collisions()
    }

    fn check_collisions(&mut self){
        let mut new_dots: Vec<Point> = vec![];
        for player in &mut self.players {
            let result = find_point(
                &self.dots,
                player.head().x,
                player.head().y
            );

            match result {
                Some(index) => {
                    self.dots.remove(index);
                    let dot = self.create_dot();
                    new_dots.push(dot);
                },
                None => {
                    player.points.remove(0);
                },
            }

            if player.head().x as usize >= self.width ||
                player.head().y < 0 ||
                player.head().y as usize >= self.height ||
                player.head().x < 0 {
                panic!("Game Over");
            }
        }

        for dot in new_dots{
            self.dots.push(dot);
        }


//        let &player_1 = self.players[0];
//        let &self.players[1] = self.players[1];

        let index = find_point(&self.players[0].points, self.players[0].head().x, self.players[0].head().y).unwrap();
        if index != self.players[0].points.len() - 1{
            panic!("Game Over");
        }

        let index = find_point(&self.players[1].points, self.players[1].head().x, self.players[1].head().y).unwrap();
        if index != self.players[0].points.len() - 1{
            panic!("Game Over");
        }

        if find_point(&self.players[1].points, self.players[0].head().x, self.players[0].head().y).is_some(){
            panic!("Game Over");
        }

        if find_point(&self.players[0].points, self.players[1].head().x, self.players[1].head().y).is_some(){
            panic!("Game Over");
        }
    }
}

fn draw(stdout: &mut RawTerminal<Stdout>, board: &Board) {
//    write!(stdout,
//       "{}",
//       termion::clear::All
//    ).unwrap();

    for y in 0..board.height {
        let mut string = "".to_string();
        for x in 0..board.width {
            if y == board.players[0].head().y as usize && x == board.players[0].head().x as usize {
                string.push_str(&board.players[0].symbol());
                if y == board.players[1].head().y as usize && x == board.players[1].head().x as usize {
                    string.push_str(&board.players[1].symbol());
                } else if find_point(&board.players[0].points, x as isize, y as isize).is_some() {
                    string.push_str("•")
                } else if find_point(&board.players[1].points, x as isize, y as isize).is_some() {
                    string.push_str("•")
                } else if find_point(&board.dots, x as isize, y as isize).is_some() {
                    string.push_str("·");
                } else {
                    string.push_str(" ");
                }
            }
            string.push_str("█");
            writeln!(stdout, "{}{}", termion::cursor::Goto(1, y as u16 + 1), string).unwrap();
        }

        writeln!(stdout, "{}{}", termion::cursor::Goto(1, board.height as u16 + 1), "█".repeat(board.width + 1)).unwrap();

        writeln!(stdout, "{}{}--{}", termion::cursor::Goto(1, board.height as u16 + 2), board.players[0].points.len() - 1, board.players[1].points.len() - 1).unwrap();

        stdout.flush().unwrap();
    }
}

fn main() {
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().keys();

    let mut board = Board{
        width: 30,
        height: 20,
        dots: vec![],
        players: vec![
            Player{
                direction: Direction::Right,
                points: vec![
                    Point {
                        x: 0,
                        y: 0,
                    }
                ],
            },
            Player{
                direction: Direction::Left,
                points: vec![
                    Point {
                        x: 29,
                        y: 19,
                    }
                ],
            },
        ]
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
                Key::Left => board.set_player_direction(Direction::Left, 1),
                Key::Right => board.set_player_direction(Direction::Right, 1),
                Key::Up => board.set_player_direction(Direction::Up, 1),
                Key::Down => board.set_player_direction(Direction::Down, 1),
                Key::Char('A') => board.set_player_direction(Direction::Left, 1),
                Key::Char('D') => board.set_player_direction(Direction::Right, 1),
                Key::Char('W') => board.set_player_direction(Direction::Up, 1),
                Key::Char('S') => board.set_player_direction(Direction::Down, 1),
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