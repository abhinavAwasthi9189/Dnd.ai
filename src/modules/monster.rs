use crate::modules::dice::Dice;
use std::io;


#[derive(Debug,Clone)]
enum montype{
    Boss,
    Skirmisher,
    Beast,
}

#[derive(Debug,Clone)]
pub struct Monster{
    name: String,
    iqbase:montype,
    attackmod:i16,
    dexteritymod:i16,
    mhp: u16, //maximum health points
    hp:u16,   // heath points
    ac:i16,   // Armor Class
}

impl Monster{
    pub fn get_mdex(&self) -> i16{
        self.dexteritymod
    }
}


