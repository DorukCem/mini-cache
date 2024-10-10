import socket
import subprocess
import time
import pytest

HOST = "127.0.0.1"
PORT = 7879
STARTUP_TIMEOUT = 5  # seconds


def is_port_open(host, port):
    """Helper function to check if the TCP port is open."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        try:
            s.connect((host, port))
            return True
        except (ConnectionRefusedError, OSError):
            return False


def wait_for_app_to_start(host, port, timeout=STARTUP_TIMEOUT):
    """Wait until the application is ready by checking if the port is open."""
    start_time = time.time()
    while time.time() - start_time < timeout:
        if is_port_open(host, port):
            print(f"Application is up and running on {host}:{port}")
            return True
        time.sleep(1)  # Wait a second before retrying
    raise TimeoutError(f"Failed to start the application within {timeout} seconds.")


@pytest.fixture(scope="module")
def start_redis_clone():
    """Fixture to start the redis_clone application before tests and stop it after."""
    # Start the application using cargo run
    process = subprocess.Popen(
        ["cargo", "run"], stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )

    try:
        # Wait for the application to be ready
        wait_for_app_to_start(HOST, PORT)
    except TimeoutError as e:
        process.terminate()
        process.wait()
        pytest.fail(str(e))  # Fail the test if the application doesn't start

    yield process  # Test will run after this

    # Stop the application once tests are done
    process.terminate()
    process.wait()


@pytest.fixture
def tcp_connection():
    """Fixture to establish and return a TCP connection."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        yield s  # Provide the socket to the test


def test_ping(start_redis_clone, tcp_connection):
    """Test to verify PING command response."""
    message = "PING"
    tcp_connection.sendall(message.encode("utf-8"))
    data = tcp_connection.recv(1024)

    # Assert the response
    assert (
        data.decode("utf-8") == "+PONG\r\n"
    ), f"Expected '+PONG\r\n', but got {data.decode('utf-8')}"


def test_double_ping(start_redis_clone, tcp_connection):
    """Test to send two PING commands and verify two +PONG responses."""
    # Send two PING commands in a row
    message = "PING"

    for _ in range(2):
        tcp_connection.sendall(message.encode("utf-8"))  # Send both PING commands
        # Receive response for both PINGs
        data = tcp_connection.recv(1024)  # Buffer size is 1024 bytes
        # Assert that the response contains two +PONG\r\n
        expected_response = "+PONG\r\n"
        assert (
            data.decode("utf-8") == expected_response
        ), f"Expected '{expected_response}', but got {data.decode('utf-8')}"
