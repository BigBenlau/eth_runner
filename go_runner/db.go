package main

import (
	"encoding/csv"
	"fmt"
	"io"
	"log"
	"os"
	"strconv"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/vm"
)

var wg sync.WaitGroup

func check(e error) {
	if e != nil {
		panic(e)
	}
}

func print_opcode_list(op_time_list map[string][]int64) {
	for op_code, time_value_list := range op_time_list {
		for time_value := range time_value_list {
			fmt.Println("Opcode name is", op_code, "Run time as nanos: ", time_value)
		}
	}
	wg.Done()
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

	total_exec_time := time.Duration(0)
	total_used_gas := uint64(0)
	total_op_count := map[string]int64{}
	total_op_time := map[string]int64{}

	f, err := os.Open("../block_range.csv")
	check(err)
	defer f.Close()

	write_file, err := os.Create("op_time_list.csv")
	check(err)
	defer write_file.Close()

	csvReader := csv.NewReader(f)
	for {
		rec, err := csvReader.Read()
		if err == io.EOF || rec[0] == "" {
			break
		}
		if err != nil {
			log.Fatal(err)
		}
		headnumber, _ := strconv.ParseUint(rec[0], 10, 64)

		fmt.Println("Headnumber is:", headnumber)
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

		startTime := time.Now()
		// _, _, usedGas, _, op_count, op_time, op_time_list, op_gas_list := bc.Processor().Process(block, statedb, vm.Config{})
		_, _, usedGas, _, _, _, op_time_list, _ := bc.Processor().Process(block, statedb, vm.Config{})
		elapsedTime := time.Since(startTime)

		trieRead := statedb.SnapshotAccountReads + statedb.AccountReads // The time spent on account read
		trieRead += statedb.SnapshotStorageReads + statedb.StorageReads // The time spent on storage read
		exec_time := elapsedTime - trieRead                             // The time spent on EVM processing

		wg.Add(1)
		go print_opcode_list(op_time_list)

		// fmt.Println("elapsedTime", elapsedTime)
		// fmt.Println("exec time", exec_time)
		// fmt.Println("usedGas", usedGas)

		// fmt.Println("db output op count", op_count)
		// fmt.Println("db output op time", op_time)

		// fmt.Fprintln(write_file, "Headnumber:", headnumber)
		// for op_code, time_value := range op_time_list {
		// 	fmt.Fprintln(write_file, "OpCode:", op_code)
		// 	fmt.Fprintln(write_file, "Time Used:", time_value)
		// 	fmt.Fprintln(write_file, "Gas Used:", op_gas_list[op_code])
		// 	total_op_count[op_code] += op_count[op_code]
		// 	total_op_time[op_code] += op_time[op_code]
		// }
		// fmt.Fprintln(write_file, "")

		total_exec_time += exec_time
		total_used_gas += usedGas
	}
	// loop finish

	fmt.Println("Total Exec Time:", total_exec_time)
	fmt.Println("Total Used Gas:", total_used_gas)

	write_file2, err2 := os.Create("opcode_average_time.csv")
	check(err2)
	defer write_file2.Close()

	for op_code, time_value := range total_op_time {
		count := total_op_count[op_code]
		avg_time := time_value / count

		fmt.Fprintln(write_file2, op_code, avg_time, count)
	}
	fmt.Println("Wait until print_opcode_list finish.")
	wg.Wait()
}

func main() {
	ReadTest3()
}
