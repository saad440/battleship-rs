#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::{HashMap, HashSet};
use rand::{
    distributions::{Distribution, Standard},
    Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use almost;
use regex::Regex;


pub struct Board {
    cells: HashMap<Position, Cell>,
    ships: HashSet<Ship>,
    n_rows: u32,
    n_cols: u32,
    game_progress: f32,
    game_complete: bool
}

pub enum BoardConfig {
    Auto,
    Manual(HashMap<ShipType, (Position, Direction)>)
}

impl BoardConfig {
    pub fn validate(&self) -> bool {
        match self {
            Self::Auto => true,
            Self::Manual(manualconf) => Self::validate_manual(manualconf)
        }
    }

    pub fn validate_manual(boardconf: &HashMap<ShipType, (Position, Direction)>) -> bool {
        let mut board = Board::new();  // Make a temporary board
        let mut occupied_cells = HashSet::<Position>::new();
        for shiptype in ShipType::iter() {
            let mut ship = Ship::new(shiptype);
            let ship_pos = boardconf.get(&shiptype);
            if let None = ship_pos {
                continue;
            }
            let (pos, dir) = ship_pos.unwrap();
            let place_result = board.place_ship_manual(&ship, pos, dir);
            if let Err(msg) = place_result {
                return false
            }
            let cells_taken = place_result.unwrap();
            occupied_cells.extend(cells_taken.clone());
            ship.cells = cells_taken;
            board.ships.insert(ship);
        }
        true
    }
}

impl Board {
    const COLS: [char;9] = ['A','B','C','D','E','F','G','H','I'];
    const ROWS: [u8;9] = [1,2,3,4,5,6,7,8,9];

    pub fn new() -> Board {
        let n_rows: u32 = 9;
        let n_cols: u32 = 9;
        let game_progress: f32 = 0.0;
        let mut cells: HashMap<Position, Cell> = HashMap::new();

        for x in 1..=n_cols as i32 {
            for y in 1..=n_rows as i32 {
                cells.insert(Position::new(x,y), Cell::new(Position::new(x,y)));
            }
        }

        let ships: HashSet<Ship> = HashSet::new();

        Board{cells:cells, ships:ships, game_complete:false, n_rows, n_cols, game_progress}
    }

    pub fn setup(&mut self, config:BoardConfig) -> Result<(), &str> {
        if let BoardConfig::Manual(ship_positions) = config {
            for (shiptype, (start_pos, dir)) in ship_positions {
                let mut ship = Ship::new(shiptype);
                let place_result = self.place_ship_manual(&ship, &start_pos, &dir);
                if let Ok(cells_taken) = place_result {
                    ship.cells = cells_taken;
                    self.ships.insert(ship);
                }
                else {
                    return Err("Invalid Position for Ship")
                }
            }
            return Ok(())
        }

        else {
            for shiptype in ShipType::iter() {
                let mut ship = Ship::new(shiptype);
                let cells_taken = self.place_ship_auto(&ship);
                ship.cells = cells_taken;
                self.ships.insert(ship);
            }
            return Ok(())
        }
    }

    pub fn contains_cell(&self, pos: &Position) -> bool {
        match self.cells.get(pos) {
            Some(_cell) => true,
            None => false
        }
    }

    pub fn get_next_pos(&self, pos:Position, dir: Direction) -> Position {
        let p = Position{
            x: pos.x + dir.x as i32,
            y: pos.y + dir.y as i32
        };
        p
    }

    pub fn get_next_cell(&self, pos:Position, dir: Direction) -> Option<(Position, &Cell)> {
        let p = Position{
            x: pos.x + dir.x as i32,
            y: pos.y + dir.y as i32
        };

        match self.cells.get(&p) {
            Some(cell) => Some((p, cell)),
            None => None
        }
    }

    pub fn is_valid_position(&self, pos: &Position) -> bool {
        ((pos.x > 0) & (pos.x < (self.n_cols as i32+1))) & ((pos.y > 0) & (pos.y < (self.n_rows as i32+1)))
    }

    pub fn get_occupied_cells(&self) -> HashSet<Position> {
        let mut occupied_cells: HashSet<Position> = HashSet::new();

        for (pos,cell) in self.cells.iter() {
            if cell.is_occupied() {
                occupied_cells.insert(pos.clone());
            }
        }
        occupied_cells
    }

    pub fn get_unoccupied_cells(&self) -> HashSet<Position> {
        let occupied_cells = self.get_occupied_cells();
        let mut unoccupied_cells: HashSet<Position> = HashSet::new();

        // First add all cells
        for pos in self.cells.keys() {
            unoccupied_cells.insert(pos.clone());
        }
        // Now remove occupied cells
        for pos in occupied_cells {
            unoccupied_cells.remove(&pos);
        }
        unoccupied_cells
    }

    pub fn place_ship_auto(&mut self, ship:&Ship) -> Vec<Position> {
        // In Progress
        let cells_needed = ship.ship_type.get_size() as usize;
        let unoccupied_cells = self.get_unoccupied_cells();
        let mut cells_taken: Vec<Position> = Vec::new();
        let mut ship_placed = false;

        while ! ship_placed {
            cells_taken.clear();  // Reset cells taken
            // Pick a random start position
            let i = rand::thread_rng().gen_range(0..unoccupied_cells.len());
            let start_pos = unoccupied_cells.iter().nth(i).unwrap().clone();
            let start_cell = self.cells.get(&start_pos).unwrap();
            if start_cell.is_occupied() {
                continue  // Start again if occupied
            }
            cells_taken.push(start_pos);
            // Pick a random direction to move in
            let dir_name: DirectionName = rand::random();
            let dir = Direction::new(dir_name);
            // Start moving
            let mut current_pos = start_pos.clone();
            for i in 1..cells_needed {
                if let None = self.get_next_cell(start_pos, dir) {
                    break;  // Fell outside the board
                }
                // Check if next cell is occupied
                let next_pos = self.get_next_pos(current_pos, dir);
                if !self.is_valid_position(&next_pos) {
                    break;  // Fell outside the board
                }
                let next_cell = self.cells.get(&next_pos).unwrap();
                if next_cell.is_occupied() {
                    break;  // Start again if occupied
                }
                current_pos = next_pos.clone();
                cells_taken.push(current_pos);
            }
            
            if cells_taken.len() == cells_needed {
                // Success
                ship_placed = true;
            }   // Otherwise start again
        }

        // Set of taken cells is finalized.
        // Now we can set them as occupied.
        for pos in cells_taken.iter() {
            let cell = self.cells.get_mut(pos).unwrap();
            cell.set_occupied();
        }
        // Return cells taken.
        cells_taken
    }

    pub fn place_ship_manual(&mut self, ship: &Ship, start_position: &Position, direction: &Direction) -> Result<Vec<Position>, &str> {
        let cells_needed = ship.ship_type.get_size() as usize;
        let start_cell = self.cells.get(start_position).unwrap();
        let mut cells_taken: Vec<Position> = Vec::new();
        let start_pos = start_position.clone();
        let dir = direction.clone();
        cells_taken.push(start_pos);
        // Start moving
        let mut current_pos = start_pos.clone();
        for i in 1..cells_needed {
            if let None = self.get_next_cell(start_pos, dir) {
                return Err("Ship fell outside the board")
            }
            // Check if next cell is occupied
            let next_pos = self.get_next_pos(current_pos, dir);
            if !self.is_valid_position(&next_pos) {
                return Err("Ship fell outside the board")
            }
            let next_cell = self.cells.get(&next_pos).unwrap();
            if next_cell.is_occupied() {
                return Err("Collision with another ship")
            }
            current_pos = next_pos.clone();
            cells_taken.push(current_pos);
        }
        // Set of taken cells is finalized.
        // Now we can set them as occupied.
        for pos in cells_taken.iter() {
            let cell = self.cells.get_mut(pos).unwrap();
            cell.set_occupied();
        }
        Ok(cells_taken)
    }

    pub fn get_contents(&self) -> [[char; 9]; 9] {
        let mut contents = [['0'; 9]; 9];
        for (pos,cell) in self.cells.iter() {
            if cell.is_occupied() {
                contents[pos.x as usize -1][pos.y as usize -1] = '1';

                if cell.was_hit_successfully() {
                    contents[pos.x as usize -1][pos.y as usize -1] = 'X';
                }
            }
        }
        contents
    }

    pub fn hit_cell(&mut self, pos:Position) -> bool {
        if ! self.is_valid_position(&pos) {
            return false
        }
        let mut hit: bool = false;
        let cell = self.cells.get_mut(&pos).unwrap();
        cell.hit();
        if cell.is_occupied() & (cell.get_hitcount()==1) {
            hit = true
        }
        self.update_status();
        hit
    }

    pub fn update_status(&mut self) {
        let mut total_cells: f32 = 0.0;
        let mut occupied_cells: f32 = 0.0;
        let mut successful_hits: f32 = 0.0;

        for cell in self.cells.values() {
            total_cells += 1.0;

            if cell.is_occupied() {
                occupied_cells += 1.0;

                if cell.was_hit_successfully() {
                    successful_hits += 1.0;
                }
            }
        }

        self.game_progress = (successful_hits/occupied_cells) * 100.0;

        if almost::equal(self.game_progress, 100.0) {
            self.game_complete = true;
        }
    }

    pub fn get_progress(&self) -> f32 {
        self.game_progress
    }

    pub fn is_game_complete(&self) -> bool {
        self.game_complete
    }

    pub fn display_board(&self) {
        let contents = self.get_contents();
        for row in contents.iter() {
            for element in row {
                print!("{} ", element);
            }
            print!("\n");
        }
    }
}


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Position {
    x: i32,
    y: i32
}

impl Position {
    pub fn new(x:i32, y:i32) -> Position {
        Position{x,y}
    }
}


#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Cell {
    position: Position,
    occupied: bool,
    hitcount: usize,
}

impl Cell {
    pub fn new(pos: Position) -> Cell {
        Cell{position: pos, occupied:false, hitcount:0}
    }

    pub fn is_occupied(&self) -> bool {
        self.occupied
    }

    pub fn set_occupied(&mut self) {
        self.occupied = true;
    }

    pub fn hit(&mut self) {
        self.hitcount += 1;
    }

    pub fn get_hitcount(&self) -> usize {
        self.hitcount
    }

    pub fn was_hit_successfully(&self) -> bool {
        self.occupied & (self.hitcount > 0)
    }
}


#[derive(Hash, Eq, PartialEq, Debug)]
pub struct Ship {
    ship_type: ShipType,
    cells: Vec<Position>,
}

impl Ship {
    pub fn new(ship_type: ShipType) -> Ship {
        Ship{ship_type, cells:Vec::new()}
    }
}


#[derive(Hash, Clone, Copy, Eq, PartialEq, Debug, EnumIter)]
pub enum ShipType {
    C5,  // Canberra-class Landing Helicopter Dock
    H4,  // Hobart-class Destroyer
    L3,  // Leeuwin-class Survey Vessel
    A2   // Armidale-class Patrol Boat
}

impl ShipType {
    pub fn get_size(&self) -> u8 {
        match self {
            Self::A2 => 2,
            Self::L3 => 3,
            Self::H4 => 4,
            Self::C5 => 5,
        }
    }
}


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, EnumIter)]
pub enum DirectionName {
    Up,
    Down,
    Left,
    Right
}

impl Distribution<DirectionName> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> DirectionName {
        match rng.gen_range(0..4) {
            0 => DirectionName::Up,
            1 => DirectionName::Down,
            2 => DirectionName::Left,
            _ => DirectionName::Right,
        }
    }
}


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Direction{
    name: DirectionName,
    x: i8,
    y: i8
}

impl Direction {
    pub fn new(dir: DirectionName) -> Direction {
        let (x,y) = match dir {
            DirectionName::Up => (0,-1),
            DirectionName::Down => (0,1),
            DirectionName::Left => (-1,0),
            DirectionName::Right => (1,0)
        };
        Direction{name:dir, x:x, y:y}
    }

    pub fn get_coord(&self) -> [i8;2] {
        [self.x, self.y]
    }

    pub fn get_name(&self) -> &DirectionName {
        &self.name
    }
    
}


#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum GameCommand {
    StartGame,
    Cell(i32,i32),
    Quit,
    InvalidCommand
}


pub fn command_parser (cmd: &str) -> GameCommand {
    // Check 1. Is it STARTGAME?
    if cmd == "STARTGAME" {
        return GameCommand::StartGame
    }
    // Check 2. Is it a cell position?
    let pattern_cell = r"^CELL:\[[0-9],[0-9]\]$";
    let re_cell = Regex::new(pattern_cell).unwrap();
    if re_cell.is_match(cmd) {
        let x: i32 = String::from(cmd.chars().nth(6).unwrap()).parse().unwrap();
        let y: i32 = String::from(cmd.chars().nth(8).unwrap()).parse().unwrap();
        return GameCommand::Cell(x,y)
    }
    // Check 3. Is it a QUIT command?
    if cmd == "QUIT" {
        return GameCommand::Quit
    }

    GameCommand::InvalidCommand
}


pub enum CommandResult {
    Success(String),
    Failure(String),
    Message(String),
    GameComplete(i32),  // TODO: Implement some kind of score to return
    Some(Board),
    None,
    Quit
}


pub fn command_handler(board: &mut Option<Board>, cmd:GameCommand) -> CommandResult {
    match cmd {
        GameCommand::StartGame => {
            let mut board_new = Board::new();
            let r = board_new.setup(BoardConfig::Auto);
            match r {
                Ok(_) => return CommandResult::Some(board_new),
                Err(_) => return CommandResult::None,
            }
        }
        GameCommand::Cell(x,y) => {
            // Make sure a board exists
            if let None = board {
                return CommandResult::Failure(String::from("No game started yet."));
            }
            let board = board.as_mut().unwrap();
            let result = board.hit_cell(Position{x,y});
            if result {
                if board.is_game_complete() {
                    return CommandResult::GameComplete(0);
                    }
                return CommandResult::Success(String::from("HIT"))
            }
            else {
                return CommandResult::Failure(String::from("MISS"))
            }
        }
        GameCommand::Quit => {
            return CommandResult::Quit
        }
        GameCommand::InvalidCommand => {
            return CommandResult::None
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn board_cell_relations() {
        let b = Board::new();
        let p = Position::new(2,2);
        let cell = b.cells.get(&p);
        let something = match cell {
            Some(cell) => true,
            None => false
        };
        assert_eq!(something, true);
    }

    #[test]
    fn commands_parsing_correctly () {
        assert_eq!(command_parser("STARTGAME"), GameCommand::StartGame);
        assert_eq!(command_parser("CELL:[3,1]"), GameCommand::Cell(3,1));
        assert_eq!(command_parser("QUIT"), GameCommand::Quit);
    }
}
