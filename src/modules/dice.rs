// dice module
use rand::RngExt;
use rand::rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u16)]
pub enum Dice {
    D4 = 4,
    D6 = 6,
    D8 = 8,
    D10 = 10,
    D12 = 12,
    D20 = 20,
    D100 = 100,
}

impl Dice {
    pub fn roll_die(&self) -> u16 {
        let mut my_rng = rng();
        let val = *self as u16;
        my_rng.random_range(1..=val)
    }

    //this functions rolls d6 4 times and adds the 3 higher values. its something in dnd, idk.
    pub fn charstat() -> u16 {
        let mut satcode: [u16; 4] = [0; 4];
        satcode[0] = Dice::D6.roll_die();
        satcode[1] = Dice::D6.roll_die();
        satcode[2] = Dice::D6.roll_die();
        satcode[3] = Dice::D6.roll_die();
        satcode.sort();
        satcode.iter().skip(1).sum()
    }
}
