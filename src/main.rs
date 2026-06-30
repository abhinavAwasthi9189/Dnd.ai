use rand;

use std::io;
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

    fn feasible(items: &[Self]) -> u8 {
        //checks if a player is alive
        let has_player = items.iter().any(|item| matches!(item, Self::Player(_)));
        
        //checks if a monster is alive
        let has_monster = items.iter().any(|item| matches!(item, Self::Mon(_)));
        
        if has_monster && has_player{return 2;}
        else if has_monster{return 0;}
        else if has_player{return 1;}
        else{3}
    }

    fn mon_index(items: &[Mob]) -> Vec<usize>{
        items.iter().enumerate().filter(|(_,mob)| matches!(mob,Mob::Mon(_))).map(|(index,_)|index).collect()
    }

    fn player_index(items: &[Mob]) -> Vec<usize> {
        //it takes mob removes all monsters, makes a list of all players and their indexes are
        //original then sorts them as per hp
        let mut players_with_indices: Vec<(&Character, usize)> = items.iter().enumerate()
        .filter_map(|(index, mob)| match mob {
            Mob::Player(player) => Some((player, index)),
            _ => None, 
        }).collect();
        players_with_indices.sort_by_key(|(player, _)| player.hp);

        // so it only sends the indexes not the whole thing
        players_with_indices.into_iter().map(|(_, index)| index).collect()
    }
    
    fn get_ac(&self) -> i16 {
        match self {
            Mob::Player(character) => character.ac,
            Mob::Mon(monster) => monster.ac,
        }
    }

    fn name(&self) -> String{
        match self {
            Mob::Player(character) => character.name.clone(),
            Mob::Mon(monster) => monster.name.clone(),
        }
    }
    

    fn get_hp(&self) -> u16 {
        match self {
            Mob::Player(character) => character.hp,
            Mob::Mon(monster) => monster.hp,
        }
    }

    fn take_damage(&mut self, amount: u16) {
        match self {
            Mob::Player(character) => {
                character.hp -= amount;
            }
            Mob::Mon(monster) => {
                monster.hp -= amount;
            }
        }
    }

    fn status(&self){
        match self {
            Mob::Player(character) => character.status(),
            Mob::Mon(monster) => monster.status(),
        }
    }
    

}

fn combat_encounter(playvec: &mut Vec<Character>, monvec: &mut Vec<Monster>){
    //number of raidplayer has taken part in.
    static mut RAIDNUM:u16 = 0;
    
    //creating and setting when each player will play
    let mut mob: Vec<Mob> = monvec.iter().cloned().map(Mob::Mon).collect();
    mob.extend(playvec.iter().cloned().map(Mob::Player));
    mob.sort_by_key(|item| item.get_dex());
    mob.reverse();

'master: loop{
        for i in 0..mob.len(){
            // which player/ monster will currently attack.
            // and play his chance
            if mob[i].get_hp() == 0 {continue;}
            let attacked :usize;let damage: u16;
            match &mob[i] {
                Mob::Player(kharacter) => {
                    println!("{}'s chance",kharacter.name);
                    println!("you will have list of monsters and their index. write value in decimal as <index>.<option>");
                    println!("option=> 1 to attack & 2 to assess");
                    loop{
                        let choice = intputf32();
                        let index = choice.trunc() as usize;
                        let option = choice.fract();
                        match option{
                            0.2 => {mob[index].status();}
                            0.1 =>{attacked=index;break;}//this may look like a issue that someone being able to atatck anyone. but its a feature. TAKE REVENGE PEOPLE REVENGE!!!! 
                            _ => {println!("wrong choice try again");}
                        }
                    }
                    let diceroll = Dice::D20.roll_die();
                    println!("dice rolls to {}",diceroll);
                    let attack_total = diceroll as i16 + kharacter.attack_modi();
                    if attack_total > mob[attacked].get_ac(){
                        println!("able to go threw the arrow attack will land");
                        let diceroll = kharacter.attack_dice().roll_die();
                        damage = diceroll + kharacter.attack_modi() as u16;
                    }
                    else{
                        println!("enemy armour too strong. unable to attack");damage=0;
                    }
                }
                Mob::Mon(monster) => {
                    match monster.iqbase{
                        monster::montype::Boss => {attacked = Mob::player_index(&mob)[0];}
                        monster::montype::Skirmisher => {let indexes = Mob::player_index(&mob); attacked = indexes[rand::random_range(0..indexes.len())]; }
                        monster::montype::Beast => {attacked = rand::random_range(0..mob.len());}
                    }
                    let diceroll = Dice::D20.roll_die();
                    println!("dice rolls to {}",diceroll);
                    let attack_total = diceroll as i16 + monster.attackmod;
                    if attack_total > mob[attacked].get_ac(){
                        println!("able to go threw the arrow attack will land");
                        let diceroll = monster.attackdie.roll_die();
                        damage = diceroll + monster.attackmod as u16;
                    }
                    else{
                        println!("enemy armour too strong. unable to attack");damage=0;
                    }      
                }
            }
            
            mob[attacked].take_damage(damage);
            println!("{} was attacked by {} damage dealth {} cuurent_hp {}",mob[attacked].name(),mob[i].name(),damage,mob[attacked].get_hp());
    
            //check if a player or a monster is left to attack or  should we just stop.
            match Mob::feasible(&mob){
                0 => {
                    break 'master;}
                1 => {
                    unsafe{RAIDNUM +=1;}
                    
                    break 'master;}
                _ => {}
            }
        }
    } 
    //copy back the updated character data
    playvec.clear();
    for entity in mob {
        if let Mob::Player(updated_character) = entity {
            playvec.push(updated_character);
        }
    }

    //if the player level increases by certain amount we level up
    if playvec[0].get_clevel()*10 < unsafe{RAIDNUM}{
        for playup in playvec.iter_mut(){
            playup.leveup();
        }
     }
}


fn main() {
    println!("Hello, world!");
}


fn intputf32() -> f32 {
    println!("enter the value");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let input: f32 = input.trim().parse().expect("Please type a number!");
    input
}
