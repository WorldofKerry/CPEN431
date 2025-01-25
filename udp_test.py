import socket
import time

def benchmark_udp_client(server_host='127.0.0.1', server_port=16401, num_requests=10000):
    client_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    start_time = time.time()

    for _ in range(num_requests):
        client_socket.sendto(b'', (server_host, server_port))
        _, _ = client_socket.recvfrom(1024)

    throughput = num_requests / (time.time() - start_time)
    print(f"Throughput = {throughput:.2f} requests/second")
    client_socket.close()

if __name__ == '__main__':
    benchmark_udp_client()
