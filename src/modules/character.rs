use crate::modules::dice::Dice;
use std::io;

#[derive(Debug,Clone)]
enum Klass{
    Figher,
    Wizard,
    Rogue,
}

#[derive(Debug,Clone)]
enum Species{
    Human,
    Elf,
    Dwarf,
}

#[derive(Debug,Copy,Clone)]
struct Stats{
    strength: i16,
    dexterity: i16,
    constitution: i16,
    intelligence: i16,
    wisdom: i16,
    charisma: i16,
}

#[derive(Debug,Clone)]
pub struct Character{
    name: String,
    klass:Klass,
    species:Species,
    level: u16,
    pub stats: Stats,
    modifier: Stats,
    mhp: u16, //maximum health points
    hp:u16,   // heath points
    ac:u16,   // Armor Class
}

impl Stats{
    fn new() -> Self{
        Self{
        strength :  Dice::charstat() as i16,
        dexterity : Dice::charstat() as i16,
        constitution : Dice::charstat() as i16,
        intelligence : Dice::charstat() as i16,
        wisdom : Dice::charstat() as i16,
        charisma : Dice::charstat() as i16,
        }
    }

    fn modifier(&mut self) -> Stats{
    let modi = Stats {
        strength :  (self.strength-10)/2,
        dexterity: (self.dexterity - 10) / 2,
        constitution: (self.constitution - 10) / 2,
        intelligence: (self.intelligence - 10) / 2,
        wisdom: (self.wisdom - 10) / 2,
        charisma: (self.charisma - 10) / 2,
            };
        return modi;
        }

}

impl Character{
    pub fn new() -> Self{
        let mut stats = Stats::new();
        let modi = stats.modifier();
        
        println!("enter your character name");
        let mut naam = String::new();
        io::stdin().read_line(&mut naam).expect("Failed to read line");

        println!("now for the species. human[1], elf[2], dwarf[3]");
        let a = intput();
        let mut speC:Species = Species::Human;
        match a{
            1 =>{stats.strength +=2;}
            2 =>{speC = Species::Elf;stats.dexterity+=2;}
            3 =>{speC = Species::Dwarf;stats.constitution+=2;}
            _ =>{}
        }

        println!("now for the Class. Fighter[1], Rogue[2], Wizard[3]");
        let a = intput(); let mhp:u16; let ac:u16;
        let mut klass:Klass = Klass::Figher;
        match a{
            1 =>{mhp = (modi.constitution) as u16 +10; ac = 16;}
            2 =>{klass = Klass::Rogue;mhp = modi.constitution as u16 +8;ac = 11 + modi.dexterity as u16;}
            3 =>{klass = Klass::Wizard;mhp = modi.constitution as u16 + 6;ac = 10 + modi.dexterity as u16;}
            _ =>{mhp = 0;ac = 0;}
        }
        
        
        let character = Character {
            name:naam,
            klass:klass,
            species:speC,
            level:1,
            stats: stats,
            modifier: modi,
            mhp: mhp,
            hp: mhp,
            ac: ac,
        };

        character.status();

        character
    }
    pub fn status(&self){
        println!("Name=> {}",self.name);
        println!("Species=> {:?}",self.species);
        println!("Class=> {:?}",self.klass); 
        println!("Level=> {}",self.level);
        println!("stats=> {:?}", self.stats);
        println!("modifier=> {:?}", self.modifier);
        println!("Max. HP=> {}",self.mhp);
        println!("HP=> {}",self.hp);
        println!("ac=> {}", self.ac);
    }

    pub fn leveup(&mut self){
        self.level+=1;
        self.mhp+=6+(self.stats.constitution) as u16;
    }

    pub fn attack_dice(&self)-> Dice{
        match self.klass{
            Klass::Figher =>Dice::D12,
            Klass::Rogue => Dice::D6,
            Klass::Wizard => Dice::D8,
        }
    }

    pub fn attack_modi(&self)-> i16{
        match self.klass{
            Klass::Figher =>self.modifier.strength,
            _ => self.modifier.dexterity,
        }
    }

    pub fn get_cdex(&self) -> i16{
        self.modifier.dexterity
    }

    pub fn get_clevel(&self) -> u16{
        self.level
    }
}



fn intput() -> u16 {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input: u16 = input.trim().parse().expect("Please type a number!");
    input
}

