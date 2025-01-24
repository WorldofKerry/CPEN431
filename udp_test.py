import socket
import time

def benchmark_udp_client(server_host='127.0.0.1', server_port=16401, num_requests=10000):
    client_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    total_data_sent = 0
    total_time = 0

    for i in range(num_requests):
        message = b''  # Data to send
        start_time = time.time()  # Start time before sending
        client_socket.sendto(message, (server_host, server_port))  # Send the message
        response, _ = client_socket.recvfrom(1024)  # Wait for the response
        end_time = time.time()  # End time after receiving the response

        round_trip_time = end_time - start_time
        total_time += round_trip_time
        total_data_sent += len(message)

    # Calculate throughput in requests per second
    throughput = num_requests / total_time
    print(f"Throughput = {throughput:.2f} requests/second")
    client_socket.close()

if __name__ == '__main__':
    benchmark_udp_client()
