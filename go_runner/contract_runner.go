package main

import (
	"fmt"
	"math/big"
	"os"
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
	contractCodePath := "../keccak.sol"
	contractCodeHex, err := os.ReadFile(contractCodePath)
	check(err)

	contractCodeBytes := common.Hex2Bytes(string(contractCodeHex))
	calldataBytes := common.Hex2Bytes("")

	zeroAddress := common.BytesToAddress(common.FromHex("0x0000000000000000000000000000000000000000"))
	callerAddress := common.BytesToAddress(common.FromHex("0x1000000000000000000000000000000000000001"))

	config := params.MainnetChainConfig
	rules := config.Rules(config.MergeNetsplitBlock, true, 1681338458)
	defaultGenesis := core.DefaultGenesisBlock()
	genesis := &core.Genesis{
		Config:     config,
		Coinbase:   defaultGenesis.Coinbase,
		Difficulty: defaultGenesis.Difficulty,
		GasLimit:   defaultGenesis.GasLimit,
		Number:     1681338457,
		Timestamp:  1681338458,
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

	fmt.Println("chainconfig:  ", evm.ChainConfig())
	_, contractAddress, _, err := evm.Create(vm.AccountRef(callerAddress), contractCodeBytes, gasLimit, new(uint256.Int))
	check(err)

	//fmt.Fprintln(os.Stdout, "hehehe")
	//fmt.Println("ok!2222")

	tx1 := types.NewTx(&types.AccessListTx{
		ChainID:  big.NewInt(1),
		Nonce:    1,
		To:       &contractAddress,
		Value:    zeroValue,
		Gas:      gasLimit,
		GasPrice: zeroValue,
		Data:     calldataBytes,
	})

	signer.Sender(tx1)
	msg, _ := core.TransactionToMessage(tx1, signer, zeroValue)
	//msg := types.NewMessage(callerAddress, &contractAddress, 1, zeroValue, gasLimit, zeroValue, zeroValue, zeroValue, calldataBytes, types.AccessList{}, false)

	snapshot := statedb.Snapshot()
	statedb.AddAddressToAccessList(msg.From)
	statedb.AddAddressToAccessList(*msg.To)

	start := time.Now()
	_, _, err, _, _, _, _ = evm.Call(vm.AccountRef(callerAddress), *msg.To, msg.Data, msg.GasLimit, new(uint256.Int))
	timeTaken := time.Since(start)

	fmt.Println("used time (nanos):", float64(timeTaken.Nanoseconds()))

	check(err)

	statedb.RevertToSnapshot(snapshot)

	parallel.Print_total_op_count_and_time()
}
