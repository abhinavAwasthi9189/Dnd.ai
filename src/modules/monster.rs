use crate::modules::dice::Dice;
use std::io;


#[derive(Debug,Clone)]
pub enum montype{
    Boss,
    Skirmisher,
    Beast,
}

#[derive(Debug,Clone)]
pub struct Monster{
    pub name: String,
    pub iqbase:montype,
    pub attackmod:i16,
    pub dexteritymod:i16,
    pub hp:u16,   // heath points
    pub ac:i16,   // Armor Class
    pub attackdie: Dice,
}

impl Monster{
    pub fn get_mdex(&self) -> i16{
        self.dexteritymod
    }
    pub fn status(&self){
        println!("Name=> {}",self.name);
        println!("type=> {:?}",self.iqbase);
        println!("Attack modifier=> {:?}",self.attackmod); 
        println!("Dexterity modifier=> {}",self.dexteritymod);
        println!("HP=> {}",self.hp);
        println!("ac=> {}", self.ac);
    }
}


