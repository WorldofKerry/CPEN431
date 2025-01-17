import socket

def interact_with_udp_server(ip, port, message):
    # Create a UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        # Send data to the server
        print(f"Sending: {message}")
        sock.sendto(message.encode(), (ip, port))

        # Receive response from the server
        data, server = sock.recvfrom(1024)
        print(f"Received: {data.decode()} from {server}")
    finally:
        sock.close()

if __name__ == "__main__":
    server_ip = "0.0.0.0"  # Replace with your server's IP if needed
    server_port = 0       # Replace with your server's port
    message = "Hello, UDP Server!"

    interact_with_udp_server(server_ip, server_port, message)
