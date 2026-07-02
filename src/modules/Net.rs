// Net module
use crate::modules::character::Character;
use crate::modules::dice::Dice;
use crate::modules::monster::Monster;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
use tokio::sync::mpsc;

//these are the ways the response be made by gemini
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialResponse {
    pub story_text: String,
    pub next_action_mode: String, // "Social", "Exploration" ,"Combat_initiation"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationResponse {
    pub story_text: String,
    pub requires_roll: bool,
    pub stat_type: String, // e.g., "Strength", "Dexterity" what stat to compare with.
    pub difficulty_class: i16, // The DC to beat
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStartResponse {
    pub story_text: String,
    pub spawned_monsters: Vec<Monster>, // what are the monsters and we can spawn them.
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BattleSummaryResponse {
    pub cinematic_story: String,
    pub historical_snippet: String,
}

//its the action mode
#[derive(Debug, Clone, PartialEq)]
pub enum GameMode {
    Social,
    Exploration,
    CombatInitiation,
    BattleSummary,
}

//so these are the parts that the api will want the response be in.
#[derive(Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
}

#[derive(Serialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Serialize)]
pub struct Part {
    pub text: String,
}

//this function is for exploration part where a dice is rolled and its result tells what value you get.
pub fn exploration_check(response: &ExplorationResponse, character: &Character) -> bool {
    let roll = Dice::D20.roll_die();

    let modifier = match response.stat_type.as_str() {
        "Strength" => character.modifier.strength,
        "Dexterity" => character.modifier.dexterity,
        "Constitution" => character.modifier.constitution,
        "Intelligence" => character.modifier.intelligence,
        "Wisdom" => character.modifier.wisdom,
        "Charisma" => character.modifier.charisma,
        _ => 0,
    };

    let total = roll as i16 + modifier;
    println!(
        "the roll {}. the DC is {}. and total is {}",
        roll, response.difficulty_class, total
    );
    wait_for_enter();
    response.difficulty_class < total
}

//this function send the data we give to it in json form to gemini.
pub async fn call_gemini(api_key: &str, prompt: String) -> Result<String, reqwest::Error> {
    let client = Client::new();

    // we are using gemini-flash-2.5
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    // this is the structure that gemini likes the request to come in/
    let payload = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
    };

    // we send the request async to gemini waiting for a response
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await?
        .json::<Value>() // Parse whatever JSON response comes back safely into a generic Value map
        .await?;

    // Extract the exact text path from gemeni's nested response
    let response_text = response["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("Error: Failed to extract story text from response.")
        .to_string();

    Ok(response_text)
}

// a function that turn any string into one of the response structs
pub fn parse_gemini_json<T: DeserializeOwned>(raw_json: &str) -> Result<T, serde_json::Error> {
    // first we need to remove the casing around it.
    let clean_json = raw_json
        .trim()
        .trim_start_matches("```json")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str::<T>(clean_json)
}

struct MemoryDumpTask {
    pub current_summary: String,
    pub logs_to_compress: Vec<String>,
}

pub struct GameHistory {
    pub logs: Vec<String>,
    pub world_summary: String,
    api_key: String,
    // The transmitter handle to send background tasks down the pipe
    tx: mpsc::UnboundedSender<MemoryDumpTask>,
}

impl GameHistory {
    // this fucntion starts the game history and logging. so here we first  place the things we want
    // inside of the summary which the ai read to know what to do.
    pub fn new(party: &[Character], api_key: String) -> Self {
        let mut player_intro = String::from("Active Adventurers in this world:\n");
        for pc in party {
            player_intro.push_str(&format!(
                "- {} (Class: {:?}, Species: {:?})\n",
                pc.name, pc.klass, pc.species
            ));
        }

        let initial_summary = format!(
            "The adventure begins in a dangerous fantasy realm.\n{}",
            player_intro
        );

        //create our async thread communication channel
        let (tx, mut rx) = mpsc::unbounded_channel::<MemoryDumpTask>();
        let worker_api_key = api_key.clone();

        //spawn the parallel background consumer loop
        //it tells the ai to compress the logs into summary making to save tokens
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let compile_logs = task.logs_to_compress.join("\n");
                let directive = format!(
                    "You are a memory compressor engine. Combine these logs into our master history paragraph cleanly.\n\
                    Existing Master Summary: {}\n\n\
                    New Logs: {}\n\n\
                    Respond ONLY with a JSON object: {{ \"updated_world_memory\": \"string text\" }}",
                    task.current_summary, compile_logs
                );

                // it runs in the background the work in main can go on as needed.
                if let Ok(raw_json) = call_gemini(&worker_api_key, directive).await {
                    if let Ok(_response) = parse_gemini_json::<SummaryResponse>(&raw_json) {
                        //remove thhe comment for testing. rest not needed.
                        //println!("\n[System Memory Monitor: Background log compression complete.]");
                    }
                }
            }
        });

        Self {
            logs: Vec::new(),
            world_summary: initial_summary,
            api_key: api_key,
            tx,
        }
    }

    // this also happens in background async
    pub fn append(&mut self, text: String) {
        self.logs.push(text);

        // if our log buffer hits 10 items,just send it to be summarize down the pipe
        if self.logs.len() >= 10 {
            let task = MemoryDumpTask {
                current_summary: self.world_summary.clone(),
                logs_to_compress: self.logs.clone(),
            };

            // Fire and forget! Send the data box to the background loop
            let _ = self.tx.send(task);

            //clear the logs
            self.logs.clear();
            //remove thhe comment for testing. rest not needed.
            //println!("\n[System Memory Monitor: Shifting logs to background compiler thread...]");
        }
    }

    // converts the summary and logs into a string to be sent as jsonn to gemini
    pub fn compile_prompt(&self, player_input: &str, instructions: &str) -> String {
        let mut history_block = String::new();
        for entry in &self.logs {
            history_block.push_str(&format!("{}\n", entry));
        }

        format!(
            "System Context/Global Summary: {}\n\n\
            Recent Conversation History:\n{}\n\
            Current Player Input: '{}'\n\n\
            Strict Formatting Directive: {}",
            self.world_summary, history_block, player_input, instructions
        )
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct SummaryResponse {
    pub updated_world_memory: String,
}

fn wait_for_enter() {
    print!("Press Enter to continue...");
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}
