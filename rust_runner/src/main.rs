use reth_db::{open_db_read_only, mdbx::DatabaseArguments, models::client_version::ClientVersion};

use reth_provider::{BlockReaderIdExt, StateProviderFactory, BlockExecutor, ProviderFactory, providers::BlockchainProvider, TransactionVariant};

use reth_primitives::{
    MAINNET, BlockId, U256
};

use reth_revm::{
    database::StateProviderDatabase,
    processor::EVMProcessor,
    EvmProcessorFactory
};
use reth_node_ethereum::EthEvmConfig;

use reth_blockchain_tree::{
    BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree, TreeExternals,
};
use reth_beacon_consensus::BeaconConsensus;

use reth_interfaces::consensus::Consensus;


use std::time::Duration;
use std::{path::Path, time::Instant};
use std::sync::Arc;
use std::fs::File;
use csv::Error;

// #[derive(Parser, Debug)]


fn main() -> Result<(), Error> {
    // Read Database Info
    // written in reth/src/commands/debug_cmd/build_block.rs (from line 147)

    let db_path_str = String::from("/home/user/common/docker/volumes/eth-docker_reth-el-data/_data/db");
    let db_path = Path::new(&db_path_str);
    let db = Arc::new(open_db_read_only(&db_path, DatabaseArguments::new(ClientVersion::default())).unwrap());

    let static_files_path_str = String::from("/home/user/common/docker/volumes/eth-docker_reth-el-data/_data/static_files");
    let static_file_path = Path::new(&static_files_path_str).to_path_buf();

    let chain_spec = MAINNET.clone();

    let provider_factory = ProviderFactory::new(db.clone(), chain_spec.clone(), static_file_path,).unwrap();

    let consensus: Arc<dyn Consensus> = Arc::new(BeaconConsensus::new(chain_spec.clone()));
    let evm_config = EthEvmConfig::default();

    let tree_externals = TreeExternals::new(
        provider_factory.clone(),
        Arc::clone(&consensus),
        EvmProcessorFactory::new(chain_spec.clone(), evm_config),
    );
    let tree = BlockchainTree::new(tree_externals, BlockchainTreeConfig::default(), None).unwrap();
    let blockchain_tree = ShareableBlockchainTree::new(tree);

    let blockchain_db =
    BlockchainProvider::new(provider_factory.clone(), blockchain_tree.clone()).unwrap();



    let mut total_exec_diff = Duration::ZERO;
    let start_time = Instant::now();

    // Execute Block by block number
    let mut round_num = 0;
    // let gas_used_sum = 0;
    // let mut exec_time_sum = Duration::new(0, 0);
    let file = File::open("../block_range.csv")?;
    let mut reader = csv::ReaderBuilder::new().has_headers(false).from_reader(file);

    for result in reader.records() {
        let record = result?;
        let new_block_num = record[0].parse::<u64>().unwrap();
        println!("Run block num: {:?}", new_block_num);

        let old_block_num = new_block_num - 1;
        let new_block = blockchain_db.block_with_senders_by_id(BlockId::from(new_block_num), TransactionVariant::WithHash).unwrap().unwrap();

        let state_provider = blockchain_db.history_by_block_number(old_block_num).unwrap();

        let mut executor = EVMProcessor::new_with_db(chain_spec.clone(), StateProviderDatabase::new(state_provider), evm_config);

        // let result = executor.execute_and_verify_receipt(&new_block, U256::ZERO, None).unwrap();

        let exec_start_time = Instant::now();

        executor.execute_transactions(&new_block, U256::ZERO).unwrap();

        let exec_end_time = Instant::now();
        let exec_diff = exec_end_time.duration_since(exec_start_time);
        total_exec_diff += exec_diff;

        // let stat = executor.stats();
        // let result = executor.take_output_state();
        // println!("Show result: {:?}", result);


        // let exec_time = stat.execution_duration;

        round_num += 1;
        // gas_used_sum += gas_used;
        // exec_time_sum += exec_time;
        println!("new block number {:?}, round: {:?}", new_block_num, round_num);
    }


    let end_time = Instant::now();

    let diff = end_time.duration_since(start_time);
    println!("Overall Duration Time is {:?} s\n", diff.as_secs_f64());
    println!("Total Execution Time is {:?} s\n", total_exec_diff.as_secs_f64());


    // let gas_per_ms = gas_used_sum / exec_time_sum.as_millis();
    // println!("Total Gas Used is {:?} \nTotal Execution Time is {:?}\n Gas Used per millisecond is {:?}", gas_used_sum, exec_time_sum, gas_per_ms);
    Ok(())
}
