extern crate termion;

use rand::Rng;
use termion::event::{Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Write, stdout, Stdout};
use std::{time, thread, env};
use termion::async_stdin;

static DEATH_PENALTY: isize = -3;

struct Snake {
    points: Vec<Point>,
    direction: Direction,
}

impl Snake {
    fn head(&self) -> Point {
        *self.points.last().unwrap()
    }

    fn body(&self) -> &[Point] {
        &self.points[..self.points.len() - 1]
    }

    fn set_direction(&mut self, direction: Direction) {
        let len = self.points.len();
        if len == 1 || self.points[len - 2] != self.head().relative(direction) {
            self.direction = direction;
        }
    }

    fn stretch(&mut self){
        let new_head = self.head().relative(self.direction);
        self.points.push(new_head);
    }

    fn shrink(&mut self){
        self.points.remove(0);
    }

    fn score(& self, is_dead: bool) -> isize{
        let penalty = if is_dead  {DEATH_PENALTY} else {0};
        (self.points.len() as isize) - 1 + penalty
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Copy, Clone)]
struct Point {
    x: isize,
    y: isize,
}

impl Point {
    fn random(width: usize, height: usize) -> Point{
        let mut rng = rand::thread_rng();
        Point {
            x: rng.gen_range(0, width) as isize,
            y: rng.gen_range(0, height) as isize,
        }
    }

    fn relative(&self, direction: Direction) -> Point {
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

fn remove_multiple<T>(vector: &mut Vec<T>, indexes: &[usize]){
    let mut i:usize = 0;
    vector.retain(|_| (!indexes.contains(&i), i += 1).0);
}

fn find_point(points: &[Point], point: Point) -> Option<usize>{
    points.iter().position(|p| *p == point)
}

fn contains_point(points: &[Point], point: Point) -> bool{
    find_point(points, point).is_some()
}

struct Board {
    width: usize,
    height: usize,
    players: Vec<Snake>,
    eggs: Vec<Point>,
}

impl Board {
    fn new_egg_position(&self) -> Point{
        loop{
            let point = Point::random(self.width, self.height);

            if !self.point_contains_egg(point) &&
                !self.point_contains_snake(point){

                return point;
            }
        }
    }

    fn add_eggs(&mut self, n: usize){
        for _ in 0..n{
            let point = self.new_egg_position();
            self.eggs.push(point);
        }
    }

    fn point_contains_snake(&self, point: Point) -> bool {
        self.players.iter().any(
            |p| contains_point(&p.points, point)
        )
    }

    fn point_contains_egg(&self, point: Point) -> bool {
        contains_point(&self.eggs, point)
    }

    fn point_contains_body(&self, point: Point, player: &Snake) -> bool {
        contains_point(player.body(), point)
    }

    fn point_contains_other_snake(&self, point: Point, player: &Snake) -> bool{
        self.players.iter().any(
            |p| !std::ptr::eq(p, player) && contains_point(&p.points, point)
        )
    }

    fn point_is_out_of_bounds(&self, point: Point) -> bool{
        point.x as usize >= self.width ||
            point.y < 0 ||
            point.y as usize >= self.height ||
            point.x < 0
    }

    fn step(&mut self){
        let mut eggs_eaten = vec![];
        let mut dead_players = vec![];

        if let [player0, player1] = &self.players[..]{
            if player0.head() == player1.head(){
                panic!(game_over_message(&*self.players, &[0, 1]))
            }
        }

        for player in &mut self.players {
            player.stretch();
            let found_egg = find_point(&self.eggs,player.head());
            match found_egg {
                Some(index) => {
                    eggs_eaten.push(index);
                },
                None => {
                    player.shrink();
                },
            }
        }

        for (i, player) in self.players.iter().enumerate() {
            let died =  self.point_is_out_of_bounds(player.head()) ||
                self.point_contains_other_snake(player.head(), player) ||
                self.point_contains_body(player.head(), player);

            if died {
                dead_players.push( i);
            }
        }

        if !dead_players.is_empty() {
            panic!(game_over_message(&self.players, &*dead_players))
        }

        remove_multiple(&mut self.eggs, &eggs_eaten);
        self.add_eggs(eggs_eaten.len());
    }
}

fn game_over_message(players: &[Snake], dead_players: &[usize]) -> String{
    if let [player0, player1] = &players[..] {
        let player0_dead = dead_players.contains(&(0 as usize));
        let player1_dead = dead_players.contains(&(1 as usize));

        let message = if player0_dead && player1_dead {
            "Both died"
        } else if player0_dead {
            "Player 1 died"
        } else {
            "Player 2 died"
        };

        let score0 = player0.score(player0_dead);
        let score1 = player1.score(player1_dead);

        let winner = if score0 > score1 {
            "Player 1 wins"
        } else if score0 < score1 {
            "Player 2 wins"
        } else if player0_dead && player1_dead {
            "Draw"
        } else if player0_dead {
            "Player 2 wins"
        } else {
            "Player 1 wins"
        };

        return format!(
            "\r\n\
            Game Over:\r\n\
            {}\r\n\
            {}\r\n\
            {}____{}\r\n",
            message,
            winner,
            score0,
            score1,
        )
    }else {
        return format!(
            "\r\n\
            Game Over:\r\n\
            {}\r\n",
            players[0].score(false),
        )
    }
}

fn draw(stdout: &mut RawTerminal<Stdout>, board: &Board) {
    for y in 0..board.height {
        let mut string = "".to_string();
        for x in 0..board.width {
            let point = Point{x: x as isize, y: y as isize};
            if board.point_contains_snake(point){
                string.push_str("██")
            }else if board.point_contains_egg(point) {
                string.push_str("◖◗");
            }else{
                string.push_str("  ");
            }
        }
        string.push_str("██");
        writeln!(stdout, "{}{}", termion::cursor::Goto(1, y as u16 + 1), string).unwrap();
    }

    writeln!(stdout, "{}{}", termion::cursor::Goto(1, board.height as u16 + 1), "██".repeat(board.width + 1)).unwrap();

   if let [player0, player1] = &board.players[..] {
        writeln!(stdout, "{}{}", termion::cursor::Goto(1, board.height as u16 + 2), player0.points.len() - 1).unwrap();
        writeln!(stdout, "{}{}", termion::cursor::Goto(board.width as u16 * 2, board.height as u16 + 2), player1.points.len() - 1).unwrap();
    }else{
        let player0 = &board.players[0];
        writeln!(stdout, "{}{}", termion::cursor::Goto(1, board.height as u16 + 2), player0.points.len() - 1).unwrap();
    }

    stdout.flush().unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let multiplayer = args.len() > 1 && args[1] == "m";

    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut stdin = async_stdin().keys();

    let width = 20;
    let height = 15;
    let reads_per_step = 20;
    let steps_per_second = 10;
    let eggs = 2;

    let mut board = Board{
        width,
        height,
        eggs: vec![],
        players: vec![
            Snake{
                direction: Direction::Right,
                points: vec![
                    Point {
                        x: 0,
                        y: 0,
                    },
                ],
            },

        ]
    };

    if multiplayer{
        board.players.push(
            Snake{
                direction: Direction::Left,
                points: vec![
                    Point {
                        x: width as isize - 1,
                        y: height as isize - 1,
                    },
                ],
            },
        )
    }

    board.add_eggs(eggs);

    let mut iteration = 0;
    write!(stdout,
       "{}{}",
       termion::clear::All,
       termion::cursor::Hide,
    ).unwrap();

    loop {
        iteration +=1;
        thread::sleep(time::Duration::from_millis(1000/steps_per_second/reads_per_step));
        let result = stdin.next();

        if let [player0, player1] = &mut board.players[..] {
            if let Some(Ok(key)) = result {
                match key {
                    Key::Char('q') => break,
                    Key::Left => player1.set_direction(Direction::Left),
                    Key::Right => player1.set_direction(Direction::Right),
                    Key::Up => player1.set_direction(Direction::Up),
                    Key::Down => player1.set_direction(Direction::Down),
                    Key::Char('a') => player0.set_direction(Direction::Left),
                    Key::Char('d') => player0.set_direction(Direction::Right),
                    Key::Char('w') => player0.set_direction(Direction::Up),
                    Key::Char('s') => player0.set_direction(Direction::Down),
                    _ => {}
                }
            }
        }else{
            let player0 = &mut board.players[0];
            if let Some(Ok(key)) = result {
                match key {
                    Key::Char('q') => break,
                    Key::Left => player0.set_direction(Direction::Left),
                    Key::Right => player0.set_direction(Direction::Right),
                    Key::Up => player0.set_direction(Direction::Up),
                    Key::Down => player0.set_direction(Direction::Down),
                    _ => {}
                }
            }
        }


        if iteration % reads_per_step == 0 {
            board.step()
        }

        draw(&mut stdout, &board);
        stdout.flush().unwrap()
    }
}