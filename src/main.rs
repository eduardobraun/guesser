use wasmer::{Instance, Module, Store, Function, imports};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_jit::JIT;
use rand::thread_rng;
use rand::Rng;

fn rand32(a: u32, b: u32) -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(a..=b)
}

fn guess(n: u32) -> i8 {
    println!("Guessed: {}", n);
    if n < 42 {
        return -1;
    }

    if n > 42 {
        return 1;
    }

    std::process::exit(0);
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/players/random/target/wasm32-unknown-unknown/debug/random.wasm"
    );

    // Reads wasm file
    let wasm_bytes = std::fs::read(wasm_path)?;

    // Creates a just-in-time engine
    let store = Store::new(&JIT::new(Cranelift::default()).engine());

    // Builds the module using our engine
    let module = Module::new(&store, wasm_bytes)?;

    let rand32 = Function::new_native(&store, rand32);
    let guess = Function::new_native(&store, guess);
    // Create an import object.
    let import_object = imports! {
        "env" => {
            "rand32" => rand32,
            "guess" => guess,
        }
    };

    // Creates a new instance of our module
    let instance = Instance::new(&module, &import_object)?;

    // Gets a refence to the `guess` function
    let turn = instance.exports.get_function("turn")?;

    // Calls and print the result of our wasm function
    loop {
        turn.call(&[])?;
    }
}
