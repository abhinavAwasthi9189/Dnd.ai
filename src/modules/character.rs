use crate::modules::dice::Dice;
use std::io;

#[derive(Debug, Clone)]
pub enum Klass {
    Figher,
    Wizard,
    Rogue,
}

#[derive(Debug, Clone, Copy)]
pub enum Species {
    Human,
    Elf,
    Dwarf,
}

#[derive(Debug, Copy, Clone)]
pub struct Stats {
    pub strength: i16,
    pub dexterity: i16,
    pub constitution: i16,
    pub intelligence: i16,
    pub wisdom: i16,
    pub charisma: i16,
}

#[derive(Debug, Clone)]
pub struct Character {
    pub name: String,
    pub klass: Klass,
    pub species: Species,
    level: u16,
    pub stats: Stats,
    pub modifier: Stats,
    mhp: u16,    //maximum health points
    pub hp: u16, // heath points
    pub ac: i16, // Armor Class
}

impl Stats {
    fn new() -> Self {
        Self {
            strength: Dice::charstat() as i16,
            dexterity: Dice::charstat() as i16,
            constitution: Dice::charstat() as i16,
            intelligence: Dice::charstat() as i16,
            wisdom: Dice::charstat() as i16,
            charisma: Dice::charstat() as i16,
        }
    }

    fn modifier(&mut self) -> Stats {
        let modi = Stats {
            strength: (self.strength - 10) / 2,
            dexterity: (self.dexterity - 10) / 2,
            constitution: (self.constitution - 10) / 2,
            intelligence: (self.intelligence - 10) / 2,
            wisdom: (self.wisdom - 10) / 2,
            charisma: (self.charisma - 10) / 2,
        };
        return modi;
    }
}

impl Character {
    pub fn new() -> Self {
        let mut stats = Stats::new();
        let modi = stats.modifier();

        println!("enter your character name");
        let naam = intputstr();

        println!("now for the species. human[1], elf[2], dwarf[3]");
        let a = intput();
        let mut spe_c: Species = Species::Human;
        match a {
            1 => {
                stats.strength += 2;
            }
            2 => {
                spe_c = Species::Elf;
                stats.dexterity += 2;
            }
            3 => {
                spe_c = Species::Dwarf;
                stats.constitution += 2;
            }
            _ => {}
        }

        println!("now for the Class. Fighter[1], Rogue[2], Wizard[3]");
        let a = intput();
        let mhp: u16;
        let ac: i16;
        let mut klass: Klass = Klass::Figher;
        match a {
            1 => {
                mhp = (modi.constitution + 10) as u16;
                ac = 16;
            }
            2 => {
                klass = Klass::Rogue;
                mhp = (modi.constitution + 8) as u16;
                ac = 11 + modi.dexterity;
            }
            3 => {
                klass = Klass::Wizard;
                mhp = (modi.constitution + 6) as u16;
                ac = 10 + modi.dexterity;
            }
            _ => {
                mhp = 0;
                ac = 0;
            }
        }

        let character = Character {
            name: naam,
            klass: klass,
            species: spe_c,
            level: 1,
            stats: stats,
            modifier: modi,
            mhp: mhp,
            hp: mhp,
            ac: ac,
        };

        character.status();

        character
    }
    pub fn status(&self) {
        println!(
            "==================================================================================="
        );
        println!("Name=> {}", self.name);
        println!("Species=> {:?}", self.species);
        println!("Class=> {:?}", self.klass);
        println!("Level=> {}", self.level);
        println!("stats=> {:?}", self.stats);
        println!("modifier=> {:?}", self.modifier);
        println!("Max. HP=> {}", self.mhp);
        println!("HP=> {}", self.hp);
        println!("ac=> {}", self.ac);
        println!(
            "=================================================================================="
        );
    }

    pub fn leveup(&mut self) {
        self.level += 1;
        self.mhp += 6 + (self.stats.constitution) as u16;
    }

    pub fn attack_dice(&self) -> Dice {
        match self.klass {
            Klass::Figher => Dice::D12,
            Klass::Rogue => Dice::D6,
            Klass::Wizard => Dice::D8,
        }
    }

    pub fn attack_modi(&self) -> i16 {
        match self.klass {
            Klass::Figher => self.modifier.strength,
            _ => self.modifier.dexterity,
        }
    }

    //these functions maybe not needed in the future. i have made these things public so.
    pub fn get_cdex(&self) -> i16 {
        self.modifier.dexterity
    }

    pub fn get_clevel(&self) -> u16 {
        self.level
    }
}

fn intput() -> u16 {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let trimmed = input.trim();

        // Prevent empty input parsing errors
        if trimmed.is_empty() {
            println!("Input cannot be empty. Please type a number:");
            continue;
        }

        match trimmed.parse::<u16>() {
            Ok(num) => return num,
            Err(_) => println!("Please type a valid number!"),
        }
    }
}

fn intputstr() -> String {
    loop {
        let mut input = String::new();

        // Read line and check how many bytes were read
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => {
                panic!("Standard input stream was closed or unavailable!");
            }
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    println!("Input cannot be empty. Please type some text:");
                    continue;
                }
                return trimmed.to_string();
            }
            Err(error) => {
                panic!("Failed to read line due to an I/O error: {}", error);
            }
        }
    }
}
