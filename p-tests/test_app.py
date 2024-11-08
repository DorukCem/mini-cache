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
                socket.sendall((f"{random_val}\r\n").encode('utf-8'))
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


try:
    process = start_process()
    print("Process Started, starting tests ...")
    test_commands(process)
    test_concurrect(process)
    print("--- All Tests Passed ---")
finally:
    process.terminate()
    process.wait()
