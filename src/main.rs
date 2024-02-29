use reth_db::open_db_read_only;
use reth_db_provider::BlockReader;
use reth_db_provider::{
    BlockExecutor, ProviderFactory
};

use reth_primitives::{
    MAINNET, BlockHashOrNumber, U256
};

use reth_revm::{
    database::StateProviderDatabase,
    processor::EVMProcessor
};

use std::path::Path;
use std::sync::Arc;
use chrono::Local;

use std::time::Duration;
use std::fs::File;
use csv::{Reader, Error};

// #[derive(Parser, Debug)]


fn main() -> Result<(), Error> {
    // Read Database Info
    let string = String::from("/home/user/common/docker/volumes/eth-docker_reth-el-data/_data/db");
    let path = Path::new(&string);
    let db = Arc::new(open_db_read_only(&path, None).unwrap());
    let chain_spec = MAINNET.clone();

    let provider = Arc::new(ProviderFactory::new(&db, chain_spec.clone()));


    let start_time = Local::now();
    println!("Start Current Time is {:?}", start_time);

    // Execute Block by block number
    let mut round_num = 0;
    let mut gas_used_sum = 0;
    let mut exec_time_sum = Duration::new(0, 0);
    let file = File::open("block_range.csv")?;
    let mut reader = Reader::from_reader(file);

    for result in reader.records() {
        let record = result?;
        let new_block_num = record[0].parse::<u64>().unwrap();
        let gas_used = record[1].parse::<u128>().unwrap();

        let old_block_num = new_block_num - 1;
        let new_block = provider.block(BlockHashOrNumber::Number(new_block_num)).unwrap().unwrap();


        let state_provider = provider.history_by_block_number(old_block_num).unwrap();

        let mut executor = EVMProcessor::new_with_db(chain_spec.clone(), StateProviderDatabase::new(state_provider));

        // let result = executor.execute_and_verify_receipt(&new_block, U256::ZERO, None).unwrap();

        executor.execute(&new_block, U256::ZERO, None).unwrap();

        let stat = executor.stats();
        println!("Show stats: {:?}", stat);
        let exec_time = stat.execution_duration;

        round_num += 1;
        gas_used_sum += gas_used;
        exec_time_sum += exec_time;
        println!("new block number {:?}, round: {:?}", new_block_num, round_num);
    }


    let end_time = Local::now();
    println!("End Current Time is {:?}", end_time);

    let diff = end_time - start_time;
    println!("Duration Time is {:?} ms\n", diff.num_milliseconds());

    let gas_per_ms = gas_used_sum / exec_time_sum.as_millis();
    println!("Total Gas Used is {:?} \nTotal Execution Time is {:?}\n Gas Used per millisecond is {:?}", gas_used_sum, exec_time_sum, gas_per_ms);
    Ok(())
}
