use crate::modules::dice::Dice;
use std::io;


#[derive(Debug)]
enum montype{
    Boss,
    Skirmisher,
    Beast,
}

struct Monster{
    name: String,
    iqbase:montype,
    attackmod:u16,
    dexteritymod:u16,
    mhp: i16, //maximum health points
    hp:i16,   // heath points
    ac:i16,   // Armor Class
}
