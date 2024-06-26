package main

import (
	"fmt"
	"math/big"
	"strconv"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/parallel"
	"github.com/ethereum/go-ethereum/params"
	"github.com/holiman/uint256"
)

func run_contract() {
	// contractCodeHex := strings.Repeat("5f5f20", 256)
	contractCodeHex := ""
	var i int64
	for i = 0; i < 256; i++ {
		hex_str := strconv.FormatInt(i, 16)
		if len(hex_str) == 1 {
			hex_str = "0" + hex_str
		}
		fmt.Println(hex_str)
		contractCodeHex += "60" + hex_str + "600053600160002050"
	}
	fmt.Println("contractCodeHex is:", contractCodeHex)

	contractCodeBytes := common.Hex2Bytes(string(contractCodeHex))

	zeroAddress := common.BytesToAddress(common.FromHex("0x0000000000000000000000000000000000000000"))
	callerAddress := common.BytesToAddress(common.FromHex("0x1000000000000000000000000000000000000001"))

	config := params.MainnetChainConfig
	rules := config.Rules(config.LondonBlock, true, 1719334350)
	defaultGenesis := core.DefaultGenesisBlock()
	genesis := &core.Genesis{
		Config:     config,
		Coinbase:   defaultGenesis.Coinbase,
		Difficulty: big.NewInt(0),
		GasLimit:   30000000,
		Number:     1719334349,
		Timestamp:  1719334350,
		Alloc:      defaultGenesis.Alloc,
	}

	statedb, err := state.New(common.Hash{}, state.NewDatabase(rawdb.NewMemoryDatabase()), nil)
	check(err)

	zeroValue := big.NewInt(0)
	gasLimit := ^uint64(30000000)

	tx := types.NewTx(&types.AccessListTx{
		ChainID:  big.NewInt(1),
		Nonce:    0,
		To:       &zeroAddress,
		Value:    zeroValue,
		Gas:      gasLimit,
		GasPrice: zeroValue,
		Data:     contractCodeBytes,
	})

	signer := types.NewEIP2930Signer(big.NewInt(1))
	signer.Sender(tx)
	createMsg, _ := core.TransactionToMessage(tx, signer, zeroValue)

	//createMsg := types.NewMessage(callerAddress, &zeroAddress, 0, zeroValue, gasLimit, zeroValue, zeroValue, zeroValue, contractCodeBytes, types.AccessList{}, false)
	statedb.Prepare(rules, callerAddress, zeroAddress, &zeroAddress, vm.ActivePrecompiles(rules), createMsg.AccessList)

	blockContext := core.NewEVMBlockContext(genesis.ToBlock().Header(), nil, &zeroAddress)
	txContext := core.NewEVMTxContext(createMsg)
	evm := vm.NewEVM(blockContext, txContext, statedb, config, vm.Config{})

	parallel.Start_channel()

	start := time.Now()
	_, _, _, err = evm.Create(vm.AccountRef(callerAddress), contractCodeBytes, gasLimit, new(uint256.Int))
	timeTaken := time.Since(start)

	fmt.Println("used time (nanos):", timeTaken.Nanoseconds())

	time.Sleep(10000)
	parallel.Print_total_op_count_and_time()

	check(err)
}
