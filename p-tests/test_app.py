import itertools
import socket
import subprocess
import threading
import time
import random

HOST = "127.0.0.1"
PORT = 8012


def start_process():
    # Start the application using cargo run
    process = subprocess.Popen(
        ["cargo", "run"], stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    time.sleep(3)

    return process  # Test will run after this


def tcp_connection() -> socket.socket:
    """Establish and return a TCP connection."""
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((HOST, PORT))
    return s


def test_commands(process):
    try:
        socket = tcp_connection()

        # Test: Set command
        socket.sendall(b"set test 0 0 4\r\n")
        socket.sendall(b"1234\r\n")
        data = socket.recv(1024)
        assert (
            data.decode("utf-8") == "STORED\r\n"
        ), f"Expected 'STORED\r\n', but got {data.decode('utf-8')}"

        # Test: Get command
        socket.sendall(b"get test\r\n")
        data = socket.recv(1024)
        expected_response = "VALUE test 0 4\r\n1234\r\nEND\r\n"
        assert (
            data.decode("utf-8") == expected_response
        ), f"Expected '{expected_response}', but got {data.decode('utf-8')}"

        # Test: Get command for non existing
        socket.sendall(b"get nonexisting\r\n")
        data = socket.recv(1024)
        expected_response = "END\r\n"
        assert (
            data.decode("utf-8") == expected_response
        ), f"Expected '{expected_response}', but got {data.decode('utf-8')}"

        # Test: Unknown command
        socket.sendall(b"unknowncmd test\r\n")
        data = socket.recv(1024)
        assert (
            data.decode("utf-8") == "ERROR\r\n"
        ), f"Expected 'ERROR\r\n', but got {data.decode('utf-8')}"

        # Test: Add command (new key)
        socket.sendall(b"add newkey 0 0 4\r\n")
        socket.sendall(b"data\r\n")
        data_add_newkey = socket.recv(1024)
        assert (
            data_add_newkey.decode("utf-8") == "STORED\r\n"
        ), f"Expected 'STORED\r\n', but got {data_add_newkey.decode('utf-8')}"

        # Test: Add command (existing key)
        socket.sendall(b"add test 0 0 4\r\n")
        socket.sendall(b"test\r\n")
        data_add_existing = socket.recv(1024)
        assert (
            data_add_existing.decode("utf-8") == "NOT_STORED\r\n"
        ), f"Expected 'NOT_STORED\r\n', but got {data_add_existing.decode('utf-8')}"

        # Test: Replace command (existing key)
        socket.sendall(b"replace test 0 0 4\r\n")
        socket.sendall(b"john\r\n")
        data_replace_existing = socket.recv(1024)
        assert (
            data_replace_existing.decode("utf-8") == "STORED\r\n"
        ), f"Expected 'STORED\r\n', but got {data_replace_existing.decode('utf-8')}"

        # Verify replaced value using get
        socket.sendall(b"get test\r\n")
        data_replace_verify = socket.recv(1024)
        expected_replace_response = "VALUE test 0 4\r\njohn\r\nEND\r\n"
        assert (
            data_replace_verify.decode("utf-8") == expected_replace_response
        ), f"Expected '{expected_replace_response}', but got {data_replace_verify.decode('utf-8')}"

        # Test: Replace command (non-existing key)
        socket.sendall(b"replace test2 0 0 4\r\n")
        socket.sendall(b"data\r\n")
        data_replace_nonexisting = socket.recv(1024)
        assert (
            data_replace_nonexisting.decode("utf-8") == "NOT_STORED\r\n"
        ), f"Expected 'NOT_STORED\r\n', but got {data_replace_nonexisting.decode('utf-8')}"


        print("---- Test Commands Passed ----")

    finally:
        socket.close()


def test_concurrect(process: subprocess.Popen[bytes]):
    threads = []

    def perform_set():
        try:
            socket = tcp_connection()

            for _ in range(10):
                random_val = random.randint(0, 9)

                socket.sendall(b"set value 0 0 1\r\n")
                socket.sendall((f"{random_val}\r\n").encode("utf-8"))
                data = socket.recv(1024)
                assert (
                    data.decode("utf-8") == "STORED\r\n"
                ), f"Expected 'STORED\r\n', but got {data.decode('utf-8')}"

        finally:
            socket.close()

    for i in range(10):
        t = threading.Thread(target=perform_set)
        threads.append(t)
        t.start()

    # Wait for all threads to complete
    for t in threads:
        t.join()

    print("---- Test Concurrent Passed ----")


def test_expiration(process):
    try:
        socket = tcp_connection()

        # Set a key with a 4-second expiration time
        socket.sendall(b"set tempkey 0 4 5\r\n")
        socket.sendall(b"hello\r\n")
        data = socket.recv(1024)
        assert (
            data.decode("utf-8") == "STORED\r\n"
        ), f"Expected 'STORED\r\n', but got {data.decode('utf-8')}"

        # Retrieve the key before expiration
        socket.sendall(b"get tempkey\r\n")
        data = socket.recv(1024)
        expected_response = "VALUE tempkey 0 5\r\nhello\r\nEND\r\n"
        assert (
            data.decode("utf-8") == expected_response
        ), f"Expected '{expected_response}', but got {data.decode('utf-8')}"

        # Wait for expiration
        time.sleep(4.1)

        # Attempt to retrieve the key after expiration
        socket.sendall(b"get tempkey\r\n")
        data = socket.recv(1024)
        expected_response = "END\r\n"
        assert (
            data.decode("utf-8") == expected_response
        ), f"Expected '{expected_response}', but got {data.decode('utf-8')}"

        print("---- Test Expiration Passed ----")

    finally:
        socket.close()


try:
    process = start_process()
    print("Process Started, starting tests ...")
    test_commands(process)
    test_concurrect(process)
    test_expiration(process)
    print("--- All Tests Passed ---")
finally:
    process.terminate()
    process.wait()
