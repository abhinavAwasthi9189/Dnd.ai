use rand;
use std::io::{self, Write};
mod modules;
use crate::modules::{
    character::Character,
    dice::Dice,
    monster::{self, Monster},
};
use modules::Net::{
    ExplorationResponse, GameHistory, GameMode, SocialResponse, call_gemini, parse_gemini_json,
};

#[derive(Debug, Clone)]
enum Mob {
    Player(Character),
    Mon(Monster),
}

impl Mob {
    //find the dexterity of both monster and players to know who will go first
    fn get_dex(&self) -> i16 {
        match self {
            Mob::Player(character) => character.get_cdex() + Dice::D20.roll_die() as i16,
            Mob::Mon(monster) => monster.get_mdex() + Dice::D20.roll_die() as i16,
        }
    }

    fn feasible(items: &[Self]) -> u8 {
        //checks if a player is alive
        let has_player = items.iter().any(|item| matches!(item, Self::Player(_)));

        //checks if a monster is alive
        let has_monster = items.iter().any(|item| matches!(item, Self::Mon(_)));

        if has_monster && has_player {
            return 2;
        } else if has_monster {
            return 0;
        } else if has_player {
            return 1;
        } else {
            3
        }
    }

    fn mon_index(items: &[Mob]) {
        for (i, mob) in items.iter().enumerate() {
            if let Mob::Mon(entity) = mob {
                if entity.hp == 0 {
                    continue;
                }
                println!("|{}|=>|{}|", entity.name, i);
            }
        }
    }

    fn player_index(items: &[Mob]) -> Vec<usize> {
        //it takes mob removes all monsters, makes a list of all players and their indexes are
        //original then sorts them as per hp
        let mut players_with_indices: Vec<(&Character, usize)> = items
            .iter()
            .enumerate()
            .filter_map(|(index, mob)| match mob {
                Mob::Player(player) => Some((player, index)),
                _ => None,
            })
            .collect();
        players_with_indices.sort_by_key(|(player, _)| player.hp);

        // so it only sends the indexes not the whole thing
        players_with_indices
            .into_iter()
            .map(|(_, index)| index)
            .collect()
    }

    fn get_ac(&self) -> i16 {
        match self {
            Mob::Player(character) => character.ac,
            Mob::Mon(monster) => monster.ac,
        }
    }

    fn name(&self) -> String {
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
                if character.hp > amount {
                    character.hp -= amount;
                } else {
                    character.hp = 0;
                    print!("{} fainted", character.name);
                }
            }
            Mob::Mon(monster) => {
                if monster.hp > amount {
                    monster.hp -= amount;
                } else {
                    monster.hp = 0;
                    print!("{} fainted", monster.name);
                }
            }
        }
    }

    fn status(&self) {
        match self {
            Mob::Player(character) => character.status(),
            Mob::Mon(monster) => monster.status(),
        }
    }
}

pub fn combat_encounter(playvec: &mut Vec<Character>, monvec: &mut Vec<Monster>) -> String {
    // ◄ Change return type here!
    // number of raids player has taken part in.
    static mut RAIDNUM: u16 = 0;

    // create our collection string to hold the battle telemetry
    let mut telemetry_log = String::from("COMBAT RESULTS LOG:\n");

    // creating and setting when each player will play
    let mut mob: Vec<Mob> = monvec.iter().cloned().map(Mob::Mon).collect();
    mob.extend(playvec.iter().cloned().map(Mob::Player));
    mob.sort_by_key(|item| item.get_dex());
    mob.reverse();

    'master: loop {
        for i in 0..mob.len() {
            if mob[i].get_hp() == 0 {
                continue;
            }
            let attacked: usize;
            let damage: u16;

            match &mob[i] {
                Mob::Player(kharacter) => {
                    println!(
                        "================={}'s chance====================",
                        kharacter.name
                    );
                    println!("write value in decimal as <index>.<option> | 1=attack, 2=assess");
                    Mob::mon_index(&mob);
                    loop {
                        let (index, option) = input_index_option();
                        match option {
                            2 => {
                                mob[index].status();
                            }
                            1 => {
                                attacked = index;
                                break;
                            }
                            _ => {
                                println!("wrong choice try again");
                            }
                        }
                    }
                    let diceroll = Dice::D20.roll_die();
                    println!("dice rolls to {}", diceroll);
                    wait_for_enter();

                    let attack_total = diceroll as i16 + kharacter.attack_modi();
                    if attack_total > mob[attacked].get_ac() {
                        println!("Attack lands!");
                        let hit_roll = kharacter.attack_dice().roll_die();
                        damage = hit_roll + kharacter.attack_modi() as u16;

                        // record successful hit into our log string
                        telemetry_log.push_str(&format!(
                            "- {} attacked {} and HIT (D20 total: {}, dealt {} damage).\n",
                            kharacter.name,
                            mob[attacked].name(),
                            attack_total,
                            damage
                        ));
                        wait_for_enter();
                    } else {
                        println!("Enemy armour too strong.");
                        damage = 0;

                        // record miss into our log string
                        telemetry_log.push_str(&format!(
                            "- {} attacked {} and MISSED (D20 total: {} vs AC {}).\n",
                            kharacter.name,
                            mob[attacked].name(),
                            attack_total,
                            mob[attacked].get_ac()
                        ));
                    }
                }
                Mob::Mon(monster) => {
                    println!(
                        "================={}'s chance====================",
                        monster.name
                    );
                    match monster.iqbase {
                        monster::Montype::Boss => {
                            attacked = Mob::player_index(&mob)[0];
                        }
                        monster::Montype::Skirmisher => {
                            let indexes = Mob::player_index(&mob);
                            attacked = indexes[rand::random_range(0..indexes.len())];
                        }
                        monster::Montype::Beast => {
                            attacked = rand::random_range(0..mob.len());
                        }
                    }
                    let diceroll = Dice::D20.roll_die();
                    println!("dice rolls to {}", diceroll);
                    wait_for_enter();

                    let attack_total = diceroll as i16 + monster.attackmod;
                    if attack_total > mob[attacked].get_ac() {
                        println!("Attack lands!");
                        let hit_roll = monster.attackdie.roll_die();
                        damage = hit_roll + monster.attackmod as u16;

                        // record monster hit
                        telemetry_log.push_str(&format!(
                            "- Monster {} attacked {} and HIT (D20 total: {}, dealt {} damage).\n",
                            monster.name,
                            mob[attacked].name(),
                            attack_total,
                            damage
                        ));
                        wait_for_enter();
                    } else {
                        println!("Armor held strong.");
                        damage = 0;

                        // record monster miss
                        telemetry_log.push_str(&format!(
                            "- Monster {} attacked {} and MISSED (D20 total: {}).\n",
                            monster.name,
                            mob[attacked].name(),
                            attack_total
                        ));
                    }
                }
            }

            // apply damage locally
            mob[attacked].take_damage(damage);
            println!(
                "{} was dealt {} damage. Current HP: {}",
                mob[attacked].name(),
                damage,
                mob[attacked].get_hp()
            );

            // log death checks immediately
            if mob[attacked].get_hp() == 0 {
                telemetry_log.push_str(&format!(
                    "* Status Warning: {} has fallen in battle!\n",
                    mob[attacked].name()
                ));
            }
            wait_for_enter();
            println!("=====================================================");

            // check final win/loss state conditions
            match Mob::feasible(&mob) {
                0 => {
                    println!("monster won");
                    telemetry_log.push_str("\nVERDICT: Battle ended. The Monsters overran the party. Total Party Wipe.");
                    break 'master;
                }
                1 => {
                    println!("heroes won");
                    unsafe {
                        RAIDNUM += 1;
                    }
                    telemetry_log
                        .push_str("\nVERDICT: Battle ended. The Heroes emerged victorious!");
                    break 'master;
                }
                _ => {}
            }
        }
    }

    // copy back the updated character data structures, so that we know if player was damaged or
    // something like that.
    playvec.clear();
    for entity in mob {
        if let Mob::Player(updated_character) = entity {
            playvec.push(updated_character);
        }
    }

    // if player gets 10+raids success. they level up. what it does is in character module
    if playvec[0].get_clevel() * 10 < unsafe { RAIDNUM } {
        telemetry_log.push_str(" [SYSTEM NOTICE: Party Leveled Up!]\n");
        for playup in playvec.iter_mut() {
            playup.leveup();
        }
    }

    // at end return back what happened in the batt;e
    telemetry_log
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .expect("Please set the GEMINI_API_KEY environment variable!");
    let mut playvec: Vec<Character> = Vec::new();
    for _ in 0..3 {
        let khar = Character::new();
        playvec.push(khar);
        wait_for_enter();
    }
    let mut history = GameHistory::new(&playvec, api_key.clone());
    let mut current_mode = GameMode::Social; // Default starting point

    println!("===============================================================");
    println!("     DND AI ENGINE ACTIVE: WELCOME TO A BRAND NEW ADVENTURE    ");
    println!("=============================================================\n");

    loop {
        // at start we input the instruction we want to do.
        print!("\nYour Action > ");
        io::stdout().flush()?;
        let mut player_input = String::new();
        io::stdin().read_line(&mut player_input)?;
        let player_input = player_input.trim();

        if player_input.eq_ignore_ascii_case("exit") {
            println!("Saving state... Goodbye adventurer!");
            break;
        }

        // for what type of input we were expecting the story moves on.
        match current_mode {
            GameMode::Social => {
                let directive = "You are a TTRPG Dungeon Master.be an active story maker, add stuff on your own, don't always wait for user suggestions. Respond ONLY with a JSON object matching this schema:\n\
    {\n\
      \"story_text\": \"Description and NPC dialogue here\",\n\
      \"next_action_mode\": \"Social\" or \"Exploration\" or \"CombatInitiation\"\n\
    }\n\
    Rules for next_action_mode:\n\
    - Set to 'Exploration' ONLY if the player attempts an uncertain action requiring a skill roll (climbing, picking locks, lifting heavy objects).\n\
    - Set to 'CombatInitiation' ONLY if violence breaks out or characters attack.\n\
    - Otherwise, default to 'Social'. Do not include markdown wraps.";

                let prompt = history.compile_prompt(player_input, directive);

                println!("----System: Processing narrative turn------");
                match call_gemini(&api_key, prompt).await {
                    Ok(raw_string) => {
                        match parse_gemini_json::<SocialResponse>(&raw_string) {
                            Ok(response) => {
                                println!("\n{}", response.story_text);
                                //putting what user and ai into the logs
                                history.append(format!("Player: {}", player_input));
                                history.append(format!("Narrator: {}", response.story_text));

                                // then we see what next action will be and we change the gamemode as per that
                                match response.next_action_mode.as_str() {
                                    "Exploration" => {
                                        current_mode = GameMode::Exploration;
                                        println!(
                                            "\n[System Notice: Transitioning to Skill Check Engine]"
                                        );
                                    }
                                    "CombatInitiation" => {
                                        current_mode = GameMode::CombatInitiation;
                                        println!(
                                            "\n[System Notice: Initiative Triggered! Transitioning to Combat Engine]"
                                        );
                                    }
                                    _ => current_mode = GameMode::Social,
                                }
                            }
                            Err(_) => println!("System Error: Narrative corruption. Retrying..."),
                        }
                    }
                    Err(e) => println!("Network drop: {:?}", e),
                }
            }
            GameMode::CombatInitiation => {
                let directive = "A combat encounter has begun! Generate the enemy combatants. Respond ONLY with a JSON object matching this schema:\n\
    {\n\
      \"story_text\": \"A brief 1-sentence battle roar or description of the ambush.\",\n\
      \"spawned_monsters\": [\n\
         { \"name\": \"String\", \"hp\": 15, \"ac\": 13, \"iqbase\": \"Skirmisher\", \"attackmod\": 2, \"dexteritymod\": 1, \"attackdie\": \"D6\" }\n\
      ]\n\
    }\n\
    Match your spawned_monsters array exactly to the type of enemies involved in the narrative scene.";

                let prompt = history.compile_prompt("", directive);

                //we go from normal mode to battle first get the monsters and then go to battle.
                println!(
                    "System: Rolling local initiative matrices and populating enemy data payload..."
                );
                match call_gemini(&api_key, prompt).await {
                    Ok(raw_string) => {
                        match parse_gemini_json::<modules::Net::CombatStartResponse>(&raw_string) {
                            Ok(response) => {
                                println!("\nNarrator: {}", response.story_text);
                                println!("\n--- ENEMIES SPAWNED ---");

                                let mut active_monsters = response.spawned_monsters.clone();

                                //we start the function for battle
                                let battlesum =
                                    combat_encounter(&mut playvec, &mut active_monsters);

                                //add the battlesumamry into the logs.
                                history.append(battlesum);

                                // then we get to battle summarization, i added it as a seprate block as we send
                                // data and then get back the result. this was easier.
                                current_mode = GameMode::BattleSummary;
                            }
                            Err(e) => {
                                println!("System Error: Failed to parse enemy blocks: {:?}", e);
                                current_mode = GameMode::Social;
                            }
                        }
                    }
                    Err(_) => current_mode = GameMode::Social,
                }
            }
            GameMode::Exploration => {
                let directive = "The player is attempting a mechanical challenge. Respond ONLY with a JSON object matching this schema: \
    { \"story_text\": \"Description of setup\", \"requires_roll\": true, \"stat_type\": \"Strength\" or \"Dexterity\" or \"Intelligence\", \"difficulty_class\": 13 } \
    Do not include markdown tags other than the JSON format itself.";

                let prompt = history.compile_prompt(player_input, directive);

                println!("----System: Evaluating terrain parameters and difficulty class----");
                match call_gemini(&api_key, prompt).await {
                    Ok(raw_string) => {
                        match parse_gemini_json::<ExplorationResponse>(&raw_string) {
                            Ok(response) => {
                                println!("\n{}", response.story_text);
                                history.append(format!("Narrator: {}", response.story_text));

                                let mut outcome_message =
                                    String::from("The player proceeds normally.");

                                if response.requires_roll {
                                    println!("\n[MECHANICAL CHALLENGE DETECTED]");

                                    println!("choose the player to do this thing");
                                    for (i, p) in playvec.iter().enumerate() {
                                        println!("{} => {}", p.name, i);
                                    }
                                    let player_id: usize = loop {
                                        print!(">");
                                        // Ensure the prompt character '>' shows up instantly
                                        let _ = std::io::Write::flush(&mut std::io::stdout());

                                        let mut input = String::new();
                                        // Replaced '?' with .expect to avoid needing an enclosing Result function
                                        std::io::stdin()
                                            .read_line(&mut input)
                                            .expect("Failed to read input");

                                        match input.trim().parse::<usize>() {
                                            Ok(idx) => {
                                                // Guard clause: Ensure the choice actually exists in your vector
                                                if idx < playvec.len() {
                                                    break idx;
                                                }
                                                println!(
                                                    "Invalid index. Choose a number between 0 and {}.",
                                                    playvec.len() - 1
                                                );
                                            }
                                            Err(_) => println!("Please type a valid number."),
                                        }
                                    };
                                    print!("\n");

                                    println!(
                                        "Challenge Type: {} Check | Difficulty Target: (DC {})",
                                        response.stat_type, response.difficulty_class
                                    );

                                    let active_character = &playvec[player_id];

                                    //  i am not explaining this again. go read it in Net module
                                    let check_passed = modules::Net::exploration_check(
                                        &response,
                                        active_character,
                                    );

                                    if check_passed {
                                        println!("System Roll: SUCCESS! You beat the DC.");
                                        outcome_message = format!(
                                            "SKILL CHECK RESULT: {} passed the {} check (DC was {}). They succeed spectacularly.",
                                            active_character.name,
                                            response.stat_type,
                                            response.difficulty_class
                                        );
                                    } else {
                                        println!(
                                            "System Roll: FAILURE! You failed to beat the DC."
                                        );
                                        outcome_message = format!(
                                            "SKILL CHECK RESULT: {} failed the {} check (DC was {}). They face an obstacle or consequence.",
                                            active_character.name,
                                            response.stat_type,
                                            response.difficulty_class
                                        );
                                    }
                                }

                                // append the calculation payload directly into history memory
                                history.append(outcome_message);

                                current_mode = GameMode::Social;

                                // for the result we give gpt and ask for a response as per that.
                                println!("----System: Generating cinematic outcome narration----");
                                let follow_up_directive = "A skill check just occurred. Take the last 'SKILL CHECK RESULT' log entry and narrate the exact cinematic consequence or reward. Respond with standard SocialResponse schema.";
                                let follow_up_prompt =
                                    history.compile_prompt("", follow_up_directive);

                                if let Ok(story_json) =
                                    call_gemini(&api_key, follow_up_prompt).await
                                {
                                    if let Ok(story_data) =
                                        parse_gemini_json::<SocialResponse>(&story_json)
                                    {
                                        println!("\nNarrator: {}", story_data.story_text);
                                        history
                                            .append(format!("Narrator: {}", story_data.story_text));

                                        // if the exploration might lead to a battle then this is used.
                                        match story_data.next_action_mode.as_str() {
                                            "CombatInitiation" => {
                                                current_mode = GameMode::CombatInitiation;
                                                println!(
                                                    "\n[System Notice: The situation escalated! Transitioning to Combat Engine]"
                                                );
                                            }
                                            "Exploration" => current_mode = GameMode::Exploration,
                                            _ => current_mode = GameMode::Social,
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!(
                                    "System Error: Failed to parse exploration parameters: {:?}",
                                    e
                                );
                                current_mode = GameMode::Social;
                            }
                        }
                    }
                    Err(e) => {
                        println!("Network dropped during check: {:?}", e);
                        current_mode = GameMode::Social;
                    }
                }
            }

            GameMode::BattleSummary => {
                // this is in the simplest word the summarizer. it reads the logs and asks gpt to give a long
                // cinematic response for us to read and a simpler summary to be placed in the history.
                let summary_directive = "The local combat engine has finished calculations. Look at the last log entry containing the 'COMBAT RESULTS LOG' math and transform it into a cinematic story conclusion. Respond ONLY with a JSON object matching this schema:\n\
    {\n\
      \"cinematic_story\": \"A gripping narrative paragraph of the killing blow or combat conclusion matching the selected genre profile.\",\n\
      \"historical_snippet\": \"A 1-sentence note for world memory (e.g., 'Defeated the bar thugs; party sustained minor bruising.')\"\n\
    }";

                let prompt = history.compile_prompt("", summary_directive);

                println!(
                    "System: Dispatching mathematical telemetry to Gemini for creative translation..."
                );
                match call_gemini(&api_key, prompt).await {
                    Ok(raw_string) => {
                        match parse_gemini_json::<modules::Net::BattleSummaryResponse>(&raw_string)
                        {
                            Ok(response) => {
                                println!(
                                    "\nNarrator (Cinematic Finish):\n{}",
                                    response.cinematic_story
                                );

                                // the data is seen here.
                                history.append(format!("Narrator: {}", response.cinematic_story));
                                history
                                    .world_summary
                                    .push_str(&format!(" {}", response.historical_snippet));

                                current_mode = GameMode::Social;
                            }
                            Err(e) => {
                                println!("System Error: Failed to parse cinematic finish: {:?}", e);
                                current_mode = GameMode::Social;
                            }
                        }
                    }
                    Err(_) => current_mode = GameMode::Social,
                }
            }
        }
    }

    Ok(())
}

fn input_index_option() -> (usize, u16) {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed = input.trim();
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() == 2 {
            let parsed_index = parts[0].parse::<usize>();
            let parsed_option = parts[1].parse::<u16>();
            if let (Ok(index), Ok(option)) = (parsed_index, parsed_option) {
                return (index, option);
            }
        }
        println!("Invalid format! Please enter as <index>.<option> (e.g., 1.1 or 0.2):");
    }
}

fn wait_for_enter() {
    print!("Press Enter to continue...");
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}
