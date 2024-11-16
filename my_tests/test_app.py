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


def send_command(socket: socket.socket, command: str, data: str = "") -> str:
    """Send a command to the server and return the response."""
    socket.sendall(command.encode("utf-8"))
    if data:
        socket.sendall(data.encode("utf-8"))
    return socket.recv(1024).decode("utf-8")


def assert_response(response: str, expected: str, message: str):
    """Assert that a response matches the expected value."""
    assert (
        response == expected
    ), f"{message}: Expected '{expected}', but got '{response}'"


def test_commands(process):
    try:
        socket = tcp_connection()

        # Test: Set command
        assert_response(
            send_command(socket, "set test 0 0 4\r\n", "1234\r\n"),
            "STORED\r\n",
            "Set command failed",
        )

        # Test: Get command
        assert_response(
            send_command(socket, "get test\r\n"),
            "VALUE test 0 4\r\n1234\r\nEND\r\n",
            "Get command failed for existing key",
        )

        # Test: Get command for non existing
        assert_response(
            send_command(socket, "get non-existing-key\r\n"),
            "END\r\n",
            "Get command failed for non existing key",
        )

        # Test: Unknown command
        assert_response(
            send_command(socket, "unknowncmd test\r\n"),
            "ERROR\r\n",
            "Response failed for unknown command",
        )

        # Test: Add command (new key)
        assert_response(
            send_command(socket, "add newkey 0 0 4\r\n", "data\r\n"),
            "STORED\r\n",
            "Add command failed when adding non existing key",
        )

        # Test: Add command (existing key)
        assert_response(
            send_command(socket, "add newkey 0 0 4\r\n", "data\r\n"),
            "NOT_STORED\r\n",
            "Add command failed when adding existing key",
        )

        # Test: Replace command (existing key)
        assert_response(
            send_command(socket, "replace test 0 0 4\r\n", "john\r\n"),
            "STORED\r\n",
            "Replace command failed when replacing existing key",
        )

        # Verify replaced value using get
        assert_response(
            send_command(socket, "get test\r\n"),
            "VALUE test 0 4\r\njohn\r\nEND\r\n",
            "Getting wrong key after replace",
        )

        # Test: Replace command (non-existing key)
        assert_response(
            send_command(socket, "replace test2 0 0 4\r\n", "data\r\n"),
            "NOT_STORED\r\n",
            "Replace command failed when replacing non-existing key",
        )

        # Test: Append command (existing key)
        assert_response(
            send_command(socket, "append test 0 0 4\r\n", "more\r\n"),
            "STORED\r\n",
            "Append command failed for existing key",
        )
        assert_response(
            send_command(socket, "get test\r\n"),
            "VALUE test 0 8\r\njohnmore\r\nEND\r\n",
            "Get command failed after append",
        )

        # Test: Prepend and Append command (existing key)
        assert_response(
            send_command(socket, "set middle 0 0 4\r\n", "data\r\n"),
            "STORED\r\n",
            "set command failed",
        )

        assert_response(
            send_command(socket, "prepend middle 0 0 3\r\n", "pre\r\n"),
            "STORED\r\n",
            "Prepend command failed for existing key",
        )
        assert_response(
            send_command(socket, "get middle\r\n"),
            "VALUE middle 0 7\r\npredata\r\nEND\r\n",
            "Get command failed after prepend",
        )

        assert_response(
            send_command(socket, "append middle 0 0 3\r\n", "end\r\n"),
            "STORED\r\n",
            "append command failed for existing key",
        )
        assert_response(
            send_command(socket, "get middle\r\n"),
            "VALUE middle 0 10\r\npredataend\r\nEND\r\n",
            "Get command failed after append",
        )

        # Test: Append command (non-existing key)
        assert_response(
            send_command(socket, "append foo 0 0 4\r\n", "test\r\n"),
            "NOT_STORED\r\n",
            "Append command failed for non-existing key",
        )

        # Test: Prepend command (non-existing key)
        assert_response(
            send_command(socket, "prepend foo 0 0 4\r\n", "test\r\n"),
            "NOT_STORED\r\n",
            "Prepend command failed for non-existing key",
        )

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

                assert_response(
                    send_command(socket, "set value 0 0 1\r\n", f"{random_val}\r\n"),
                    "STORED\r\n",
                    "Failed store in concurrent test case",
                )
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
        assert_response(
            send_command(socket, "set tempkey 0 4 5\r\n", "hello\r\n"),
            "STORED\r\n",
            "Set command failed",
        )

        # Retrieve the key before expiration
        assert_response(
            send_command(socket, "get tempkey\r\n"),
            "VALUE tempkey 0 5\r\nhello\r\nEND\r\n",
            "Cannot find temp key before it expires",
        )

        # Wait for expiration
        time.sleep(4.1)

        # Attempt to retrieve the key after expiration
        assert_response(
            send_command(socket, "get tempkey\r\n"),
            "END\r\n",
            "Temp key is not deleted after expiration time",
        )

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