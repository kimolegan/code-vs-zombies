use std::io;

use rand::seq::SliceRandom;
use rand::thread_rng;

const MAX_X: i32 = 16000;
const MAX_Y: i32 = 9000;
const EMPTY_POINT: Point = Point { x: 0, y: 0 };

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Human {
    id: i32,
    pos: Point,
}

impl Point {
    pub fn range(&self, other: &Point) -> i32 {
        self.range_float(other) as i32
    }

    pub fn range_float(&self, other: &Point) -> f32 {
        f32::sqrt(((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f32)
    }

    pub fn move_to(&mut self, target: &Point, distance: f32) {
        if self.range(target) as f32 <= distance {
            self.x = target.x;
            self.y = target.y;
            return;
        }

        let direction = Vector::from(self, target).as_direction();
        self.move_to_dir(&direction, distance);
    }

    pub fn move_to_dir(&mut self, direction: &Direction, distance: f32) {
        self.x = (self.x as f32 + direction.x * distance).trunc() as i32;
        self.y = (self.y as f32 + direction.y * distance).trunc() as i32;
    }
}

struct Vector {
    pub x: i32,
    pub y: i32,
}

impl Vector {
    pub fn from(start: &Point, end: &Point) -> Self {
        Vector {
            x: end.x - start.x,
            y: end.y - start.y,
        }
    }

    pub fn as_direction(&self) -> Direction {
        let length = f32::sqrt((self.x * self.x + self.y * self.y) as f32);
        Direction {
            x: self.x as f32 / length,
            y: self.y as f32 / length,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Direction {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Zombie {
    pub id: i32,
    pub pos: Point,
    pub target: Point,
}

impl Zombie {
    pub fn choose_target(&mut self, humans: &[Human], player: &Human) {
        let closest_target = humans
            .iter()
            .chain(Some(player))
            .min_by_key(|x| x.pos.range(&self.pos))
            .unwrap();

        self.target = closest_target.pos;
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
struct State {
    pub player: Human,
    pub humans: Vec<Human>,
    pub zombies: Vec<Zombie>,
    pub score: i64,
}

struct ActionGen {
    pub actions: [Direction; 8],
    possible_actions: [Direction; 9],
    rng: rand::rngs::ThreadRng,
}

impl ActionGen {
    pub fn new() -> Self {
        ActionGen {
            actions: [Direction { x: 0.0, y: 0.0 }; 8],
            possible_actions: [
                Direction {
                    x: 0.70710677,
                    y: 0.70710677,
                },
                Direction { x: 1.0, y: 0.0 },
                Direction {
                    x: 0.70710677,
                    y: -0.70710677,
                },
                Direction { x: 0.0, y: 1.0 },
                Direction { x: 0.0, y: 0.0 },
                Direction { x: 0.0, y: -1.0 },
                Direction {
                    x: -0.70710677,
                    y: 0.70710677,
                },
                Direction { x: -1.0, y: 0.0 },
                Direction {
                    x: -0.70710677,
                    y: -0.70710677,
                },
            ],
            rng: thread_rng(),
        }
    }

    pub fn gen(&mut self) {
        for action in &mut self.actions {
            *action = *self.possible_actions.choose(&mut self.rng).unwrap();
        }
    }
}

impl State {
    pub fn zombie_choose_targets(&mut self) {
        for zombie in &mut self.zombies {
            zombie.choose_target(&self.humans, &self.player);
        }
    }

    pub fn next(&mut self, dir: &Direction, player_dist: f32) {
        let zombies_before = self.zombies.len();
        let humans_before = self.humans.len();

        // zombies choose targets
        self.zombie_choose_targets();

        // zombies move
        for zombie in &mut self.zombies {
            zombie.pos.move_to(&zombie.target, 400.0);
        }

        // player moves
        self.player.pos.move_to_dir(dir, player_dist);

        // player kills zombies
        let player = self.player;
        self.zombies
            .retain_mut(|zombie| zombie.pos.range_float(&player.pos) > 2000.0);

        // zombie kill humans
        for zombie in &self.zombies {
            self.humans.retain(|human| human.pos != zombie.pos);
        }

        // calc score
        if self.humans.is_empty() {
            self.score = i64::MIN;
        } else {
            self.score += ((zombies_before - self.zombies.len()).pow(2)) as i64;
            self.score -= ((humans_before - self.humans.len()) * 1000) as i64;
        }
    }
}

const OUTPUT_ENABLED: bool = false;

macro_rules! output_if {
    ($($t:tt)*) => {
        if OUTPUT_ENABLED {
            eprintln!($($t)*);
        }
    };
}

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

fn from_input() -> State {
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    // output_if!("{input_line}");
    let inputs = input_line.split(' ').collect::<Vec<_>>();
    let x = parse_input!(inputs[0], i32);
    let y = parse_input!(inputs[1], i32);
    let player: Human = Human {
        id: -1,
        pos: Point { x, y },
    };
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    // output_if!("{input_line}");
    let human_count = parse_input!(input_line, i32);
    let mut humans = Vec::with_capacity(human_count as usize);
    for _ in 0..human_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        // output_if!("{input_line}");
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let human_id = parse_input!(inputs[0], i32);
        let human_x = parse_input!(inputs[1], i32);
        let human_y = parse_input!(inputs[2], i32);
        humans.push(Human {
            id: human_id,
            pos: Point {
                x: human_x,
                y: human_y,
            },
        });
    }
    let mut input_line = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    // output_if!("{input_line}");
    let zombie_count = parse_input!(input_line, i32);
    let mut zombies = Vec::with_capacity(zombie_count as usize);
    let mut zombies_next = Vec::with_capacity(zombie_count as usize);
    for _i in 0..zombie_count as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        // output_if!("{input_line}");
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let zombie_id = parse_input!(inputs[0], i32);
        let zombie_x = parse_input!(inputs[1], i32);
        let zombie_y = parse_input!(inputs[2], i32);
        let zombie_xnext = parse_input!(inputs[3], i32);
        let zombie_ynext = parse_input!(inputs[4], i32);
        zombies.push(Zombie {
            id: zombie_id,
            pos: Point {
                x: zombie_x,
                y: zombie_y,
            },
            target: EMPTY_POINT,
        });
        zombies_next.push(Zombie {
            id: zombie_id,
            pos: Point {
                x: zombie_xnext,
                y: zombie_ynext,
            },
            target: EMPTY_POINT,
        });
    }

    State {
        player,
        humans,
        zombies,
        score: 0,
    }
}

fn find_solution(state: State, action_gen: &mut ActionGen) -> Point {
    let min_human_dist = state
        .humans
        .iter()
        .map(|human| human.pos.range(&state.player.pos))
        .min()
        .unwrap_or(0);

    let min_zombie_dist = state
        .zombies
        .iter()
        .map(|zombie| zombie.pos.range(&state.player.pos))
        .min()
        .unwrap_or(0);

    let mut steps = 1;
    if min_human_dist > 5000 || min_zombie_dist > 5000 {
        steps = 5;
    }

    let mut player_dist = 500.0;
    if min_human_dist > 1000 {
        player_dist = 1000.0
    }

    let from = std::time::Instant::now();
    let mut best_score = i64::MIN;
    let mut best_action: Direction = Direction { x: 0.0, y: 0.0 };
    let mut test_state;
    let mut sims = 0;
    let mut ways = 0;
    while from.elapsed().as_millis() < 95 {
        test_state = state.clone();
        action_gen.gen();
        ways += 1;
        for action in &action_gen.actions {
            sims += steps;
            for _ in 0..steps {
                test_state.next(action, player_dist);
            }

            if test_state.score == i64::MIN {
                break;
            }
        }

        // output_if!("actions: {:?}, state: {:?}", action_gen.actions, test_state);

        if test_state.score > best_score {
            best_score = test_state.score;
            best_action = *action_gen.actions.first().unwrap();
        }
    }

    // eprintln!("ways {ways} sims: {sims}");

    let mut output = state.player.pos;
    output.move_to_dir(&best_action, player_dist);
    output.x = output.x.clamp(0, MAX_X);
    output.y = output.y.clamp(0, MAX_Y);
    output
}

/**
 * Save humans, destroy zombies!
 **/
fn main() {
    let mut action_gen = ActionGen::new();

    // game loop
    loop {
        let state = from_input();
        let output = find_solution(state, &mut action_gen);
        println!("{} {}", output.x, output.y); // Your destination coordinates
    }
}

#[test]
fn test_move() {
    let mut zombie = Point { x: 1500, y: 6251 };
    let human = Point { x: 0, y: 4500 };
    zombie.move_to(&human, 400.0);
    assert!(zombie.x == 1239);
    assert!(zombie.y == 5947);
}

#[test]
fn test_gen() {
    let action_gen = ActionGen::new();
    eprint!("{:?}", action_gen.possible_actions);
}

#[test]
fn test_case() {
    let player = Human {
        id: -1,
        pos: Point { x: 1157, y: 427 },
    };
    let humans = vec![Human {
        id: 0,
        pos: Point { x: 8000, y: 4500 },
    }];

    let zombies = vec![
        Zombie {
            id: 0,
            pos: Point { x: 5411, y: 5358 },
            target: Point { x: 0, y: 0 },
        },
        Zombie {
            id: 1,
            pos: Point { x: 10580, y: 5357 },
            target: Point { x: 0, y: 0 },
        },
    ];
    let mut action_gen = ActionGen::new();
    let state = State {
        player,
        humans,
        zombies,
        score: 0,
    };
    find_solution(state, &mut action_gen);
    // let s = "8000 2000
    // 1
    // 0 8000 4500
    // 2
    // 0 5411 5358 5790 5232
    // 1 10580 5357 10200 5230";
}
