import socket

def start_udp_server(host='127.0.0.1', port=16401):
    server_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    server_socket.bind((host, port))
    print(f"Server listening on {host}:{port}")

    while True:
        message, client_address = server_socket.recvfrom(1024)  # Receive the message (1024 bytes)
        server_socket.sendto(message, client_address)

if __name__ == '__main__':
    start_udp_server()
