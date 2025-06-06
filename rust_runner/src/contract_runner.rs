use std::{str::FromStr, thread, time::{Duration, Instant}};

use reth_primitives::{Bytes, B256};
use revm_interpreter::{
    analysis::to_analysed, opcode::InstructionTables, primitives::{Bytecode, CancunSpec, Env}, Contract, Interpreter, SharedMemory,
    start_channel, print_records
};

use reth_revm::{primitives::result, EvmBuilder, Handler};

pub use reth_revm::revm::precompile::hash::sha256_run;

extern crate alloc;

/// Revolutionary EVM (revm) runner interface
const CONTRACT_ADDRESS: B256 = B256::new([1; 32]);

pub fn run_contract_code() {
    let mut contract_str = String::from("");

    // // hex_str from 00 to ff
    // for i in 0..256 {
    //     let hex_str = format!("{:x}", i);
    //     let mut left_str = String::from("");
    //     if hex_str.len() == 1 {
    //         left_str = String::from("0");
    //     }
    //     let bytecode_each = String::from("60") + &left_str + &hex_str + &String::from("600053600160002050");
    //     contract_str.push_str(&bytecode_each);
    // }

    // hex_str all 00
    for _ in 0..256 {
        let hex_str = String::from("00");
        let bytecode_each = String::from("60") + &hex_str + &String::from("600053600160002050");
        contract_str.push_str(&bytecode_each);
    }
    println!("show contract_str: {:?}", contract_str);

    let contract_code: Bytes = Bytes::from_str(&contract_str).unwrap();

    // Set up the EVM with a database and create the contract
    let env = Env::default();

    let bytecode = to_analysed(Bytecode::new_raw(contract_code));

    // revm interpreter. (rakita note: should be simplified in one of next version.)
    let contract = Contract::new_env(&env, bytecode, CONTRACT_ADDRESS);
    let shared_memory = SharedMemory::new();

    let evm_builer = EvmBuilder::default();
    let mainnet = Handler::mainnet::<CancunSpec>();
    let builder = evm_builer.with_handler(mainnet);
    let mut evm = builder.build();

    let table = evm.handler.take_instruction_table().expect("Instruction table should be present");
    let mut interpreter = Interpreter::new(contract, u64::MAX, false);

    let _ = start_channel();

    match &table {
        InstructionTables::Plain(table) => {
            let timer = Instant::now();
            interpreter.run(shared_memory, &table,  &mut evm);
            let dur = timer.elapsed();
            println!("{}", dur.as_nanos());
        },
        InstructionTables::Boxed(table) => {
            let timer = Instant::now();
            interpreter.run(shared_memory, &table,  &mut evm);
            let dur = timer.elapsed();
            println!("{}", dur.as_nanos());
        },
    }

    // 確保channel能完成所有工作
    thread::sleep(Duration::from_secs(3));

    // 打印每個opcode運行總時間
    print_records();
}





pub fn run_precompile_hash() {
    let mut contract_str_1000 = String::from("");
    for _ in 0..100 {
        let bytecode_each = String::from("01010101010101010101");
        contract_str_1000.push_str(&bytecode_each);
    }
    for count in 0..1000 {
        let mut contract_str = String::from("");
        for _ in 0..count {
            contract_str.push_str(&contract_str_1000);
        }
        let contract_code: Bytes = Bytes::from_str(&contract_str).unwrap();

        let mut sha2_result: (u64, Bytes) = (u64::MIN, Bytes::default());
        let timer = Instant::now();
        for _ in 0..1000 {
            sha2_result = sha256_run(&contract_code, 30000000u64).unwrap();
        }
        let dur = timer.elapsed().as_nanos();
        println!("Show Byte len: {:?}. Used gas: {:?}. Used time: {:?}ns, average time: {:?}", contract_code.len(), sha2_result.0, dur, dur / 1000);
    }
}