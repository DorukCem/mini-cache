import socket
import subprocess
import threading
import time
import pytest

HOST = "127.0.0.1"
PORT = 8012


def start_process():
    """Fixture to start the redis_clone application before tests and stop it after."""
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


def test_ping():
    """Test to verify PING command response."""
    try:
        process = start_process()
        socket = tcp_connection()

        message = "PING"
        socket.sendall(message.encode("utf-8"))
        data = socket.recv(1024)

        # Assert the response
        assert (
            data.decode("utf-8") == "+PONG\r\n"
        ), f"Expected '+PONG\r\n', but got {data.decode('utf-8')}"

    finally:
        socket.close()
        
        # Stop the application once tests are done
        process.terminate()
        process.wait()
