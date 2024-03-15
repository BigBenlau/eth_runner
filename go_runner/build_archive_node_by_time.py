import subprocess
import requests
import time
import os
import signal
import atexit

def terminate_process(process):
    if process.poll() is None:
        os.killpg(process.pid, signal.SIGINT)
        while process.poll() is None:
            print('Waiting for process to terminate...')
            time.sleep(1)

def run_geth_until_target(datadir, target_block_number, http_port, is_archive):
    print(f'{datadir}/prysm.sh beacon-chain --execution-endpoint={datadir}/geth.ipc --checkpoint-sync-url=https://sync.invis.tools --genesis-beacon-api-url=https://sync.invis.tools')

    # Geth command
    # 你要是用archive一定要注意，state存储要用hashscheme
    if is_archive:
        geth_command = f'geth --datadir {datadir} --state.scheme --syncmode full --gcmode archive --port 9000 --authrpc.port 8552 --http --http.port {http_port} --http.api "eth,debug"'
    else:
        geth_command = f'geth --datadir {datadir} --state.scheme --syncmode full --port 9000 --authrpc.port 8552 --http --http.port {http_port} --http.api "eth,debug"'

    # Open a file for each log
    geth_logfile = open(f'{datadir}/geth.log', 'w')

    # Start geth process
    geth_process = subprocess.Popen(geth_command, shell=True, start_new_session=True, stdout=geth_logfile, stderr=subprocess.STDOUT)

    # Register the termination function to be called on exit
    atexit.register(terminate_process, geth_process)

    # Sleep for 60 seconds to give geth some time to start up
    time.sleep(60)

    # Record the start time
    start_time = time.time()

    try:
        # Ethereum JSON-RPC endpoint
        rpc_endpoint = f'http://localhost:{http_port}'

        while True:
            # JSON-RPC payload
            payload = {
                "method": "eth_blockNumber",
                "params": [],
                "id": 1,
                "jsonrpc": "2.0"
            }

            # Send request
            response = requests.post(rpc_endpoint, json=payload)

            # Parse response
            current_block_number = int(response.json()['result'], 16)

            # Check if an hour has passed since the last print
            if int((time.time() - start_time) / 3600) > 0:
                print('Current block number:', current_block_number)
                # Update the start time
                start_time = time.time()

            # Check if current block number has reached target
            if current_block_number >= target_block_number:
                # Send SIGINT (Ctrl+C) to the geth process
                os.killpg(geth_process.pid, signal.SIGINT)
                print('Target block number reached, geth process stopping.')

                # Wait for geth process to terminate
                while geth_process.poll() is None:
                    print('Waiting for geth process to terminate...')
                    time.sleep(1)

                print('Geth process has terminated.')
                break

            # Sleep for a while before next request
            time.sleep(5)

    except Exception as e:
        print('Error:', str(e))

    finally:
        # If geth process is still running when script exits, make sure it is stopped
        if geth_process.poll() is None:
            geth_process.terminate()

        while geth_process.poll() is None:
            print('Waiting for geth process to terminate...')
            time.sleep(1)

# Datadir and HTTP port for each run
datadir = '/mnt/ssd1/geth_archive_2022'

# Target block numbers
target_block_number_1 = 13916166
target_block_number_2 = 16308190
http_port = 29545

# Run geth with the first settings until the first target block number is reached
run_geth_until_target(datadir, target_block_number_1 - 300, http_port, False)

# Run geth with the second settings until the second target block number is reached
run_geth_until_target(datadir, target_block_number_2 + 300, http_port, True)


#