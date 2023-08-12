use std::{io::{BufReader, BufRead}, fs::File, collections::{VecDeque, HashMap}};


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Amphipod {

    Amber,
    Bronze,
    Copper,
    Desert
}

impl Amphipod {

    fn get_destination_cup_index(&self) -> usize {
        match self {
            Self::Amber => 0,
            Self::Bronze => 1,
            Self::Copper => 2,
            Self::Desert => 3
        }
    }

    fn get_cost_per_move(&self) -> u32 {
        match self {
            Self::Amber => 1,
            Self::Bronze => 10,
            Self::Copper => 100,
            Self::Desert => 1000,   
        }
    }
}

impl TryFrom<char> for Amphipod {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Result::Ok(Amphipod::Amber),
            'B' => Result::Ok(Amphipod::Bronze),
            'C' => Result::Ok(Amphipod::Copper),
            'D' => Result::Ok(Amphipod::Desert),
            _ => Result::Err(())
        }
    }
}

#[derive(Clone)]
struct Room {

    placeholders: [Option<Amphipod>; 7],
    cups: [Cup; 4]
}

#[derive(Clone, Debug)]
struct Cup {
    capacity: usize,
    content: VecDeque<Amphipod>
}

impl Cup {

    fn new(capacity: usize) -> Self {
        Self{capacity, content: VecDeque::new()}
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn is_full(&self) -> bool {
        self.content.len() == self.capacity
    }
}


impl Room {

    const PLACEHOLDER_POSITIONS: [u32; 7] = [0, 1, 3, 5, 7, 9, 10];
    const CUP_POSITIONS: [u32; 4] = [2, 4, 6, 8];


    fn is_ordered(&self) -> bool {

        for (index, cup) in self.cups.iter().enumerate() {
            if cup.content.len() != cup.capacity || cup.content.iter().any(|a| a.get_destination_cup_index() != index) {
                return false;
            }
        }
        return true;
    }

    fn move_amphipod_to_placeholder(&mut self, origin_cup_index: usize, placeholder: usize) -> u32 {

        let origin_cup = self.cups.get_mut(origin_cup_index).unwrap();

        let amphipod = origin_cup.content.pop_front().unwrap();
        self.placeholders[placeholder] = Some(amphipod);

        let horizontal_distance = Self::PLACEHOLDER_POSITIONS[placeholder].abs_diff(Self::CUP_POSITIONS[origin_cup_index]);
        let vertical_distance = origin_cup.capacity - origin_cup.content.len();
        let distance =  vertical_distance as u32 + horizontal_distance;
        return distance * amphipod.get_cost_per_move();
    }

    fn move_amphipod_from_placeholder_to_destination(&mut self, placeholder: usize, destination_cup_index: usize) -> u32 {

        let destination_cup = self.cups.get_mut(destination_cup_index).unwrap();

        let amphipod = self.placeholders[placeholder].take().unwrap();
        destination_cup.content.push_front(amphipod);

        let horizontal_distance = Self::PLACEHOLDER_POSITIONS[placeholder].abs_diff(Self::CUP_POSITIONS[destination_cup_index]);
        let vertical_distance = 1 + destination_cup.capacity - destination_cup.content.len();
        let distance =  vertical_distance as u32 + horizontal_distance;
        return distance * amphipod.get_cost_per_move();
    }

    fn move_amphipod_from_origin_to_destination(&mut self, origin_cup_index: usize, destination_cup_index: usize) -> u32 {

        let origin_cup = self.cups.get_mut(origin_cup_index).unwrap();
        let amphipod = origin_cup.content.pop_front().unwrap();

        let destination_cup = self.cups.get_mut(destination_cup_index).unwrap();
        destination_cup.content.push_front(amphipod);

        let destination_cup = self.cups.get(destination_cup_index).unwrap();
        let origin_cup =  self.cups.get(origin_cup_index).unwrap();

        let horizontal_distance = Self::CUP_POSITIONS[origin_cup_index].abs_diff(Self::CUP_POSITIONS[destination_cup_index]);
        let vertical_distance = origin_cup.capacity - origin_cup.content.len() + destination_cup.capacity - destination_cup.content.len() + 1;
        let distance =  vertical_distance as u32 + horizontal_distance;
        return distance * amphipod.get_cost_per_move();
    }

    fn get_available_placeholders(&self, cup_number: usize) -> Vec<usize> {

        let mut res = Vec::new();

        // check left
        for index in (0..cup_number + 2).rev() {
            if self.placeholders[index] == None {
                res.push(index);
            } else {
                break;
            }
        }

        // check right
        for index in cup_number + 2..self.placeholders.len() {
            if self.placeholders[index] == None {
                res.push(index);
            } else {
                break;
            }
        }

        res
    }
    
    fn check_destination(&self, destination_cup_index: usize) -> bool {

        let cup = &self.cups[destination_cup_index];

        return !cup.is_full() && cup.content.iter().all(
            |amphipod| amphipod.get_destination_cup_index()==destination_cup_index);
    }

    fn check_path_placeholder_destination(&self, placeholder: usize, destination_cup: usize) -> bool {

        if placeholder < destination_cup + 1 {
            for placeholder_to_check in placeholder + 1..destination_cup + 2 {
                if self.placeholders[placeholder_to_check] == None {
                    continue;
                } 
                return false;
            }
            return true;
        } else if placeholder + 1 > destination_cup {
            for placeholder_to_check in destination_cup + 2..placeholder {
                if self.placeholders[placeholder_to_check] == None {
                    continue;
                } 
                return false;
            }
            return true;
        }
        return true;
    }

    fn check_path_origin_destination(&self, origin_cup: usize, destination_cup: usize) -> bool {

        if origin_cup < destination_cup {
            for placeholder_to_check in origin_cup + 2..destination_cup + 2 {
                if self.placeholders[placeholder_to_check] != None {
                    return false;
                } 
            }
            return true;
        } else if origin_cup > destination_cup {
            for placeholder_to_check in destination_cup + 2..origin_cup + 2 {
                if self.placeholders[placeholder_to_check] != None {
                    return false;
                } 
            }
            return true;
        }
        return true;
    }

}

#[derive(Clone)]
struct WalkState {
    room: Room,
    cost: u32
}

impl WalkState {

    fn get_next_states(&self) -> Vec<WalkState> {

        let mut res = Vec::new();

        for (cup_index, cup) in self.room.cups.iter().enumerate() {

            if cup.content.iter().all(|a| a.get_destination_cup_index()==cup_index) {
                continue;
            }

            let available_placeholders = self.room.get_available_placeholders(cup_index);

            for available_placeholder in available_placeholders {
                let mut new_state = self.clone();

                new_state.cost += new_state.room.move_amphipod_to_placeholder(
                    cup_index, available_placeholder);
                res.push(new_state);
            }
        }

        return res;
    }

    fn progress(&mut self) {
        while self.try_progress() {}
    }

    fn try_progress(&mut self) -> bool {

        let mut res = false;

        for cup_index in 0..self.room.cups.len() {
            if let Some(amphipod) = self.room.cups[cup_index].content.get(0) {
                if cup_index != amphipod.get_destination_cup_index()
                && self.room.check_destination(amphipod.get_destination_cup_index())
                && self.room.check_path_origin_destination(cup_index, amphipod.get_destination_cup_index()) {
                    self.cost += self.room.move_amphipod_from_origin_to_destination(
                        cup_index, amphipod.get_destination_cup_index());
                        res = true;
                }
            }
        }

        for placeholder_index in 0..self.room.placeholders.len() {
            if let Some(amphipod) = self.room.placeholders[placeholder_index] {
                if self.room.check_destination(amphipod.get_destination_cup_index())
                && self.room.check_path_placeholder_destination(placeholder_index, amphipod.get_destination_cup_index()) {
                    self.cost += self.room.move_amphipod_from_placeholder_to_destination(
                        placeholder_index, amphipod.get_destination_cup_index());
                        res = true;
                }
            }
        }

        res
    }

    fn project_costs(&self) -> u32 {

        let mut res = self.cost;

        for (placeholder_index, placeholder) in self.room.placeholders.iter().enumerate() {
            if let Some(amphipod) = placeholder {
                let length = 1 + Room::PLACEHOLDER_POSITIONS[placeholder_index].abs_diff(
                    Room::CUP_POSITIONS[amphipod.get_destination_cup_index()]);
                res += length * amphipod.get_cost_per_move();
            }
        }

        for (cup_index, cup) in self.room.cups.iter().enumerate() {
            for amphipod in cup.content.iter() {
                if cup_index != amphipod.get_destination_cup_index() {
                    let length = 2 + Room::CUP_POSITIONS[cup_index].abs_diff(
                        Room::CUP_POSITIONS[amphipod.get_destination_cup_index()]);
                    res += length * amphipod.get_cost_per_move();
                }
            }
        }

        res
    }

}

fn simulate_ordering(init_room: &Room) -> u32 {
   
    let init_state = WalkState{room: init_room.clone(), cost: 0};

    let mut stack: VecDeque<WalkState> = VecDeque::new();
    stack.push_back(init_state);

    let mut min_score = u32::MAX;

    while let Some(current_state) = stack.pop_back() {
        
        if min_score < current_state.cost {
            continue;
        } else if min_score < current_state.project_costs() {
            continue;
        }

        let mut intermediate_state = current_state.clone();
        intermediate_state.progress();

        if intermediate_state.room.is_ordered() {
            min_score = std::cmp::min(min_score, intermediate_state.cost);
            println!("Found solution. Cost: {}", min_score);
            continue;
        }
        
        intermediate_state.get_next_states().iter().for_each(|s| stack.push_back(s.clone()));
    }

    min_score
}


fn main() {

    let lines: Vec<String> = BufReader::new(File::open("input.txt").unwrap()).lines().map(|l| l.unwrap()).collect();
    let depth = lines.len() - 3;

    let mut cups = Vec::new();

    (0..4).for_each(|cup_index| {
        cups.push((0..depth).fold(Cup::new(depth), |mut cup, d| {
            cup.content.push_back(lines[2 + d].chars().nth(cup_index * 2 + 3).unwrap().try_into().unwrap());
            cup
        }))
    });

    let room = Room { placeholders: [Option::None; 7], cups: cups.try_into().unwrap()};

    println!("Simulate result {}", simulate_ordering(&room));
}
