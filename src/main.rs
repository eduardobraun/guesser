use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::error::Error;
use std::fmt;
use rand::thread_rng;
use rand::Rng;
use wasmer::{Instance, Module, Store, Function, FunctionType, Type, Value, imports};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_jit::JIT;

#[derive(Debug)]
struct GameError(String);

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: {}", self.0)
    }
}

impl Error for GameError {}

fn rand32(a: u32, b: u32) -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(a..=b)
}

pub enum GuessResult {
    Match,
    Lower,
    Higher
}

pub struct GuessingGame {
    number: u32,
    guessed: Arc<AtomicBool>
}

impl GuessingGame {
    pub fn new(number: u32) -> GuessingGame {
        GuessingGame{
            number,
            guessed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn guess(&self, n: u32) -> GuessResult {
        println!("Guessed: {}", n);
        if n < self.number {
            return GuessResult::Lower;
        }

        if n > self.number {
            return GuessResult::Higher;
        }

        self.guessed.store(true, Ordering::SeqCst);

        GuessResult::Match
    }

    pub fn run(&self, player: &Player) -> Result<(), Box<dyn Error>> {
        while !self.guessed.load(Ordering::SeqCst) {
            player.turn()?;
        }
        Ok(())
    }
}

fn compile_module(store: &Store, player: &str) -> Result<Module, Box<dyn Error>> {
    match player {
        "random" => {
            let wasm_path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/players/random/target/wasm32-unknown-unknown/debug/random.wasm"
            );
            let wasm_bytes = std::fs::read(wasm_path)?;
            let module = Module::new(&store, wasm_bytes)?;
            Ok(module)
        },
        _ => Err(Box::new(GameError("invalid module name".to_string()))),
    }
}

pub struct Player {
    instance: Instance,
}

impl Player {
    pub fn new(store: &Store, module: &Module, game: Arc<GuessingGame>) -> Result<Player, Box<dyn Error>> {
        let rand32 = Function::new_native(&store, rand32);
        let game = game.clone();
        // Dynamic function capturing our game instance
        let guess_sig = FunctionType::new(vec![Type::I32], vec![Type::I32]);
        let guess = Function::new(&store, &guess_sig, move |args| {
            let n = args[0].unwrap_i32() as u32;
            let res = match game.guess(n) {
                GuessResult::Lower => -1,
                GuessResult::Higher => 1,
                GuessResult::Match => 0,
            };

            Ok(vec![Value::I32(res)])
        });
        // Create an import object.
        let import_object = imports! {
            "env" => {
                "rand32" => rand32,
                "guess" => guess,
            }
        };

        // Creates a new instance of our module
        let instance = Instance::new(&module, &import_object)?;

        Ok(Player{
            instance,
        })
    }

    pub fn turn(&self) -> Result<(), Box<dyn Error>> {
        // Gets a refence to the `guess` function
        let turn = self.instance.exports.get_function("turn")?;
        // Calls the wasm function while not guessed
        turn.call(&[])?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Creates a just-in-time engine
    let store = Store::new(&JIT::new(Cranelift::default()).engine());
    // Compiles our wasm module
    let module = compile_module(&store, "random")?;

    let game = Arc::new(GuessingGame::new(42));
    let player = Player::new(&store, &module, game.clone())?;

    // Run ou game using the wasm player
    game.run(&player)
}
