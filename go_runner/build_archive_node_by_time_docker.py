import subprocess
import requests
import time
from datetime import datetime
import os
import signal
import atexit

def terminate_process(process, is_archive):
    if process.poll() is None:
        if is_archive:
            geth_down_command = f'docker compose -f geth_archive.yml down'
        else:
            geth_down_command = f'docker compose -f geth.yml down'
        print("Function Terminate block num show geth down command: %s" % geth_down_command)
        subprocess.Popen(geth_down_command, shell=True, start_new_session=True, stderr=subprocess.STDOUT)
        while process.poll() is None:
            print('Waiting for process to terminate...')
            time.sleep(1)

def run_geth_until_target(datadir, target_block_number, is_archive):

    # Geth command
    # 你要是用archive一定要注意，state存储要用hashscheme
    if is_archive:
        geth_command = f'docker compose -f geth_archive.yml up'
    else:
        geth_command = f'docker compose -f geth.yml up'

    # Open a file for each log
    geth_logfile = open(f'{datadir}/geth.log', 'w')

    # Start geth process
    print("geth command is: %s" % geth_command)
    geth_process = subprocess.Popen(geth_command, shell=True, start_new_session=True, stdout=geth_logfile, stderr=subprocess.STDOUT)

    # Register the termination function to be called on exit
    atexit.register(terminate_process, geth_process, is_archive)

    # Sleep for 60 seconds to give geth some time to start up
    print("Before Sleep")
    time.sleep(60)

    # Record the start time
    start_time = time.time()
    marked_start_time = datetime.now()
    print("Start time: %s" % marked_start_time)

    try:
        # Ethereum JSON-RPC endpoint
        rpc_endpoint = f'http://localhost:8545'

        while True:
            # JSON-RPC payload
            payload = {
                "method": "eth_blockNumber",
                "params": [],
                "id": 1,
                "jsonrpc": "2.0"
            }

            # Send request
            print("Start Request")
            response = requests.post(rpc_endpoint, json=payload)

            # Parse response
            current_block_number = int(response.json()['result'], 16)
            print(current_block_number)

            # Check if an hour has passed since the last print
            if int((time.time() - start_time) / 3600) > 0:
                print('Current block number:', current_block_number)
                # Update the start time
                start_time = time.time()

            # Check if current block number has reached target
            if current_block_number >= target_block_number:
                # Send docker compose down to the geth process
                if is_archive:
                    geth_down_command = f'docker compose -f geth_archive.yml down'
                else:
                    geth_down_command = f'docker compose -f geth.yml down'
                print("Target block num show geth down command: %s" % geth_down_command)
                subprocess.Popen(geth_down_command, shell=True, start_new_session=True, stderr=subprocess.STDOUT)
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
        print("final")
        if geth_process.poll() is None:
            if is_archive:
                geth_down_command = f'docker compose -f geth_archive.yml down'
            else:
                geth_down_command = f'docker compose -f geth.yml down'
            print("Finally show geth down command: %s" % geth_down_command)
            subprocess.Popen(geth_down_command, shell=True, start_new_session=True, stderr=subprocess.STDOUT)

        while geth_process.poll() is None:
            print('Waiting for geth process to terminate...')
            time.sleep(1)

# Datadir and HTTP port for each run
datadir = '/home/user/data/ethereum/eth-docker'

# Target block numbers
target_block_number_1 = 18000000
target_block_number_2 = 19300000

# Run geth with the first settings until the first target block number is reached
run_geth_until_target(datadir, target_block_number_1 - 300, False)

# Run geth with the second settings until the second target block number is reached
run_geth_until_target(datadir, target_block_number_2 + 300, True)

