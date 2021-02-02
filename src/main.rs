use wasmer::{Instance, Module, Store, Function, FunctionType, Type, Value, imports};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_jit::JIT;
use rand::thread_rng;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn rand32(a: u32, b: u32) -> u32 {
    let mut rng = thread_rng();
    rng.gen_range(a..=b)
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

    // Shared stop condition
    let guessed = Arc::new(AtomicBool::new(false));
    let guessed_closure = guessed.clone();

    // Dynamic function capturing our atomic
    let guess_sig = FunctionType::new(vec![Type::I32], vec![Type::I32]);
    let guess = Function::new(&store, &guess_sig, move |args| {
        let n = args[0].unwrap_i32() as u32;
        println!("Guessed: {}", n);
        if n < 42 {
            return Ok(vec![Value::I32(-1)]);
        }

        if n > 42 {
            return Ok(vec![Value::I32(1)]);
        }

        guessed_closure.store(true, Ordering::SeqCst);

        Ok(vec![Value::I32(0)])
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

    // Gets a refence to the `guess` function
    let turn = instance.exports.get_function("turn")?;

    // Calls the wasm function while not guessed
    while !guessed.load(Ordering::SeqCst) {
        turn.call(&[])?;
    }
    Ok(())
}
