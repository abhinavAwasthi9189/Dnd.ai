mod modules;
use crate::modules::{character::{self, Character}, dice::Dice, monster::{self, Monster}};

#[derive(Debug,Clone)]
enum Mob{
    Player(Character),
    Mon(Monster),
}

impl Mob{
    //find the dexterity of both monster and players to know who will go first
    fn get_dex(&self) -> i16 {
        match self{
            Mob::Player(character) => character.get_cdex() + Dice::D20.roll_die() as i16,
            Mob::Mon(monster) => monster.get_mdex() + Dice::D20.roll_die() as i16,
        }
    }

    pub fn feasible(items: &[Self]) -> u8 {
        //checks if a player is alive
        let has_player = items.iter().any(|item| matches!(item, Self::Player(_)));
        
        //checks if a monster is alive
        let has_monster = items.iter().any(|item| matches!(item, Self::Mon(_)));
        
        if has_monster && has_player{return 2;}
        else if has_monster{return 0;}
        else if has_player{return 1;}
        else{3}
    }
}

fn combat_encounter(playvec: &mut Vec<Character>, monvec: &mut Vec<Monster>){
    //number of raidplayer has taken part in.
    static mut RAIDNUM:u16 = 0;
    
    let mut mob: Vec<Mob> = monvec.iter().cloned().map(Mob::Mon).collect();
    mob.extend(playvec.iter().cloned().map(Mob::Player));
    mob.sort_by_key(|item| item.get_dex());
'master: loop{
        for i in 0..mob.len(){
            // which player/ monster will currently attack.
            // and play his chance
            match &mut mob[i] {
                Mob::Player(character) => {}
                Mob::Mon(monster) => {}
            }
    
            //check if a player or a monster is left to attack or  should we just stop.
            match Mob.feasable(&mob){
                0 => {
                    break 'master;}
                1 => {
                    unsafe{RAIDNUM +=1;}
                    //if the player level increases by certain amount we level up
                    if playvec[0].get_clevel()*10 < unsafe{RAIDNUM}{}
                    for playup in playvec{
                        playup.leveup();
                    }
                    break 'master;}
                _ => {}
            }
        }
    }   
}


fn main() {
    println!("Hello, world!");
}
