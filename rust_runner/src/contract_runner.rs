use std::{str::FromStr, thread, time::{Duration, Instant}, fmt::LowerHex};

use reth_primitives::{Bytes, B256};
use revm_interpreter::{
    analysis::to_analysed, opcode::InstructionTables, primitives::{Address, Bytecode, CancunSpec, Env}, Contract, Interpreter, SharedMemory,
    start_channel, print_records
};

use reth_revm::{EvmBuilder, Handler};

extern crate alloc;

/// Revolutionary EVM (revm) runner interface
const CONTRACT_ADDRESS: B256 = B256::new([1; 32]);

pub fn run_contract_code() {
    let mut contract_str = "";
    for i in 0..256 {
        let mut hex_str = format!("{:x}", i);
        let mut left_str = String::from("");
        if hex_str.len() == 1 {
            left_str = String::from("0");
        }
        let mut bytecode_each = String::from("60") + &left_str + &hex_str + &String::from("600053600160002050");
        contract_str = &(contract_str + &bytecode_each);
    }

    let contract_code: Bytes = Bytes::from_str(contract_str).unwrap();

    // Set up the EVM with a database and create the contract
    let mut env = Env::default();

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
