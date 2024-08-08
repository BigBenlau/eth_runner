use reth_db::{open_db_read_only, mdbx::DatabaseArguments, models::client_version::ClientVersion};

use reth_provider::{BlockReaderIdExt, StateProviderFactory, ProviderFactory, TransactionVariant,
    providers::{BlockchainProvider, StaticFileProvider}
};

use reth_chainspec::MAINNET;

use reth_primitives::{
    BlockId, U256
};

use reth_revm::database::StateProviderDatabase;

use reth_evm::execute::{
    Executor, BlockExecutionOutput
};

use reth_evm_ethereum::execute::EthExecutorProvider;

use reth_blockchain_tree::{
    BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree, TreeExternals,
};

use reth_beacon_consensus::EthBeaconConsensus;

use reth_consensus::{
    Consensus, PostExecutionInput
};

use revm_interpreter::{
    start_channel, print_records
};

use std::time::Duration;
use std::{path::Path, time::Instant};
use std::sync::Arc;
use std::fs::File;
use csv::Error;
use std::thread;

// pub mod contract_runner;
// use contract_runner::run_contract_code;

// #[derive(Parser, Debug)]

// const PARALLEL_STATEROOT: bool = false;

fn run_block() -> Result<(), Error> {
    // Read Database Info
    // written in bin/reth/src/commands/debug_cmd/build_block.rs (from line 147)

    let db_path_str = String::from("/home/user/common/docker/volumes/eth-docker_reth-el-data/_data/db");
    let db_path = Path::new(&db_path_str);
    let db = Arc::new(open_db_read_only(&db_path, DatabaseArguments::new(ClientVersion::default())).unwrap());

    let static_files_path_str = String::from("/home/user/common/docker/volumes/eth-docker_reth-el-data/_data/static_files");
    let static_file_path = Path::new(&static_files_path_str).to_path_buf();
    let static_file_provider = StaticFileProvider::read_only(static_file_path).unwrap();

    let chain_spec = MAINNET.clone();

    let provider_factory = ProviderFactory::new(db.clone(), chain_spec.clone(), static_file_provider);

    let consensus: Arc<dyn Consensus> = Arc::new(EthBeaconConsensus::new(chain_spec.clone()));

    let tree_externals = TreeExternals::new(
        provider_factory.clone(),
        Arc::clone(&consensus),
        EthExecutorProvider::mainnet(),
    );
    let tree = BlockchainTree::new(tree_externals, BlockchainTreeConfig::default(), None).unwrap();
    let blockchain_tree = Arc::new(ShareableBlockchainTree::new(tree));

    let blockchain_db =
    BlockchainProvider::new(provider_factory.clone(), blockchain_tree.clone()).unwrap();


    // 创建一个通道
    let _ = start_channel();

    let mut total_exec_diff = Duration::ZERO;
    let mut total_post_validation_diff = Duration::ZERO;
    let mut total_merkle_dur = Duration::ZERO;
    let start_time = Instant::now();

    // Execute Block by block number
    let mut round_num = 0;
    // let gas_used_sum = 0;


    let file = File::open("../block_range.csv")?;
    let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(file);

    for result in reader.records() {
        let record = result?;
        let new_block_num = record[0].parse::<u64>().unwrap();

        let old_block_num = new_block_num - 1;
        let new_block = blockchain_db.block_with_senders_by_id(BlockId::from(new_block_num), TransactionVariant::WithHash).unwrap().unwrap();

        let state_provider = blockchain_db.history_by_block_number(old_block_num).unwrap();
        let state_provider_db = StateProviderDatabase::new(state_provider);

        let executor = EthExecutorProvider::mainnet().eth_executor(state_provider_db);

        // execution
        let exec_start_time = Instant::now();

        // executor.execute_without_verification(&new_block, U256::ZERO).unwrap();
        let state = executor.execute((&new_block, U256::ZERO).into()).unwrap();

        let exec_diff = exec_start_time.elapsed();
        total_exec_diff += exec_diff;

        let BlockExecutionOutput { state, receipts, requests, .. } = state;

        // do post validation
        let val_start_time = Instant::now();
        consensus.validate_block_post_execution(&new_block, PostExecutionInput::new(&receipts, &requests)).ok();
        let val_dur = val_start_time.elapsed();
        total_post_validation_diff += val_dur;

        // calculate and check state root
        let state_provider_2 = blockchain_db.latest().unwrap();
        let merkle_start = Instant::now();
        let state_root = state_provider_2.state_root(&state);

        let merkle_dur = merkle_start.elapsed();
        total_merkle_dur += merkle_dur;

        println!("Show block state_root: {:?}", state_root);
        round_num += 1;
        // gas_used_sum += gas_used;

        println!("Current block num: {:?}, round: {:?}, exec_time: {:?}, valiation_time: {:?}, merkle_time: {:?}", new_block_num, round_num, exec_diff, val_dur, merkle_dur);
    }

    // 確保channel能完成所有工作
    thread::sleep(Duration::from_secs(3));

    // 打印每個opcode運行總時間
    print_records();

    let diff = start_time.elapsed();
    println!("Overall Duration Time is {:?} s", diff.as_secs_f64());
    println!("Total Execution Time is {:?} s", total_exec_diff.as_secs_f64());
    println!("Total Post Validation Time is {:?} s", total_post_validation_diff.as_secs_f64());
    println!("Total Merkleization Time is {:?} s\n", total_merkle_dur.as_secs_f64());

    // let gas_per_ms = gas_used_sum / exec_time_sum.as_millis();
    // println!("Total Gas Used is {:?} \nTotal Execution Time is {:?}\n Gas Used per millisecond is {:?}", gas_used_sum, exec_time_sum, gas_per_ms);
    Ok(())
}


fn main() {
    run_block().unwrap();
    // // run_contract_code();
    // run_precompile_hash()
}