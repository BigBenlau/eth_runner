package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"log"
	"os"
	"strconv"
	"time"

	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/parallel"
)

func check(e error) {
	if e != nil {
		panic(e)
	}
}

func ReadTest3() {
	datadir := "/home/user/common/docker/volumes/cp1_eth-docker_geth-eth1-data/_data/geth/chaindata"
	// datadir := "/home/user/data/ben/cp1_eth-docker_geth-eth1-data/_data/geth/chaindata"
	ancient := datadir + "/ancient"
	db, err := rawdb.Open(
		rawdb.OpenOptions{
			Directory:         datadir,
			AncientsDirectory: ancient,
			Ephemeral:         true,
		},
	)
	if err != nil {
		fmt.Println("rawdb.Open err!", err)
	}

	fmt.Println("start get bc")

	// 用數據庫中的數據重新建數據鏈
	// // datadir: cp_eth-docker
	// bc, _ := core.NewBlockChain(db, core.DefaultCacheConfigWithScheme(rawdb.PathScheme), nil, nil, ethash.NewFaker(), vm.Config{}, nil, nil)

	// datadir: cp1_eth-docker
	bc, _ := core.NewBlockChain(db, core.DefaultCacheConfigWithScheme(rawdb.HashScheme), nil, nil, ethash.NewFaker(), vm.Config{}, nil, nil)
	fmt.Println("get bc")

	// headhash := rawdb.ReadHeadHeaderHash(db)
	// headnumber_adr := rawdb.ReadHeaderNumber(db, headhash)
	// headnumber := *headnumber_adr

	total_exec_elapsedTime := time.Duration(0)
	total_exec_time := time.Duration(0)
	total_used_gas := uint64(0)
	parallel.Start_channel()

	round_count := uint64(0)

	f, err := os.Open("../block_range.csv")
	check(err)
	defer f.Close()

	csvReader := csv.NewReader(f)

	run_start_time := time.Now()
	for {
		rec, err := csvReader.Read()
		if err == io.EOF || rec[0] == "" {
			break
		}
		if err != nil {
			log.Fatal(err)
		}
		headnumber, _ := strconv.ParseUint(rec[0], 10, 64)

		fmt.Println("Headnumber is:", headnumber, "round idx is: ", round_count)
		round_count += 1

		parentnumber := headnumber - 1
		hashtest := rawdb.ReadCanonicalHash(db, headnumber)
		parenthash := rawdb.ReadCanonicalHash(db, parentnumber)
		block := rawdb.ReadBlock(db, hashtest, headnumber)

		if block == nil {
			log.Fatal("Failed to retrieve the latest block header")
		}
		parentblock := rawdb.ReadBlock(db, parenthash, parentnumber)
		if parentblock == nil {
			log.Fatal("Failed to retrieve the latest block header")
		}

		parentRoot := parentblock.Root()
		statedb, _ := bc.StateAt(parentRoot)
		if statedb == nil {
			log.Fatal("Failed to retrieve the statedb of parentRoot")
		}

		exec_startTime := time.Now()
		// _, _, usedGas, _, op_count, op_time, op_time_list, op_gas_list := bc.Processor().Process(block, statedb, vm.Config{})
		_, _, usedGas, _, _, _, _, _ := bc.Processor().Process(block, statedb, vm.Config{})
		exec_elapsedTime := time.Since(exec_startTime)

		trieRead := statedb.SnapshotAccountReads + statedb.AccountReads // The time spent on account read
		trieRead += statedb.SnapshotStorageReads + statedb.StorageReads // The time spent on storage read
		exec_time := exec_elapsedTime - trieRead                        // The time spent on EVM processing

		total_exec_elapsedTime += exec_elapsedTime
		total_exec_time += exec_time
		total_used_gas += usedGas
	}
	// loop finish

	run_elapsedTime := time.Since(run_start_time)

	// print records
	parallel.Print_total_op_count_and_time()

	fmt.Println("Total Run Loop Time:", run_elapsedTime)
	fmt.Println("Total Elapsed Time:", total_exec_elapsedTime)
	fmt.Println("Total Exec Time:", total_exec_time)
	fmt.Println("Total Used Gas:", total_used_gas)

}

func main() {
	ReadTest3()
}
