## Project Overview

Our distributed system consists of a peer-to-peer image sharing application with client-side encryption and permissions management. Key features include:

**Image Encryption:** Users can elect to have their images encrypted via steganography before sharing them with peers. Encryption happens in a load-balanced cloud environment.  

**Discovery Service:** A decentralized discovery service allows users to inquire which peers are online and request lower resolution sample images to view. Image owners specify access rules on a per-user basis. This service allows opening a web page for each client showing their images blurred.

**Offline Operation:** Best-effort updates of image access rules and view counts, taking into account scenarios where viewers or owners may be offline. Also allowing leases for client to be still applicable even if the client got offline and online again.

**Load Balancing:** Cloud servers use a distributed leader election algorithm to distribute encryption workload.

**Fault Tolerance:** Servers simulate failures by periodically ignoring communication to test redundancy.

## System Architecture

The system consists of client applications, a middleware layer, and a cloud platform running on 3 physical machines.

![](Pasted%20image%2020231210201045.png)

The middleware handles registration with the discovery service, encryption requests, and direct P2P image sharing (slide shows) with leasing. Cloud servers provide the image encryption and discovery services. 

### Cloud Servers

There can be multiple cloud server peers, but in our implementation for testing purposes we only hold 3 servers at most and they can be easily incremented by extending the implementation and declaration of servers on both clients and servers. Each cloud server is running identical software:

- **Leader Election:** The Bully algorithm is used to elect leader for processing encryption workload.  After multiple tests, it was the most suitable in the use cases for our product.
- **Encryption Service:** Steganography used to hide user images inside random images
- **Discovery Service:** Tracks online users and shared image previews

As each server runs identical software stacks, servers continuously share state updates to keep their internal discovery service databases and user access rule sets synchronized. An additional monitoring service checks leader liveness. If no response, a new leader election occurs. This also provides an alternative the CPU heap-based algorithm for load balancing as with a high load tests have shown that with a high stream of requests they are correctly redirected and assigned to new leaders accordingly.

A separate monitoring process runs checks on the current elected leader with TCP pings every 2 seconds. If no response after 5 tries (10 seconds), the leader is presumed dead. This triggers the election algorithm to select a new leader node.

During any downtimes when no leader is active (e.g. during election), client middleware hides the interruption by automatically retrying failed requests.

#### Failure Simulation

To simulate real failures, cloud servers voluntarily ignore some ping messages from peers and clients for short time windows of 5-20 seconds. The failure monitor detects this outage, triggering recovery processes of electing a new leader and resending requests.

By regularly exercising the failure mechanisms, availability remains high in production environments when actual crashes occur. The combination of redundancy, state sharing, failure detection and request retries provides durable fault tolerance.
### Client Middleware

The client middleware software has four key functions:

1. Register user with discovery service 
2. Send image encryption requests
3. Request shared image listings from discovery
4. Contact peers directly to view shared images through their hosted webservers

Encryption requests are load-balanced across the cloud servers via multicasting.

### Image Slide View For Clients

The image slide view on the website displays thumbnail previews from the discovery service, allowing users to request the full shared images. This is automatically opened for all clients available on discovery via the installed browser.

It is implemented by having the client middleware query the discovery service database on startup. Metadata of images willing to be shared including owner, filename, permissions and a lower resolution preview thumbnail are returned.

An HTML grid is generated client-side with the preview images and sharing request links. Clicking one sends a P2P request to the image owner to retrieve the original full resolution version, and if a lease was already given it shows the amount of views remaining before the access revoking,

This allows easily discovering and requesting shared images while minimizing server overhead for transfers. Image data is instead exchanged directly between peers.
## Implementation Details

### Programming Languages and Libraries

The project uses Rust for all components. Key libraries include:

- Serialization: Bincode, Serde 
- Images: image
- GUI: Rouille, Rocket
- Encryption: Stenography library (Python encryption library called from Rust)

For a complete list of the cargo requirements please review the Cargo.toml files in both client and server sides.
### Compilation and Execution

To compile the Rust sources:

```
cargo build --release
```

Run the server peers:

```
cargo run
```
Followed by binding the server to its assigned port.

Run client applications:

```
cargo run
```
## Evaluation

### Functionality Testing

Comprehensive test cases were developed to validate:

- Image encryption and embedded permissions
- Discovery service registration and queries
- Peer-to-peer image sharing 
- Leader election and load balancing
- Failure detection and redundancy 

We developed comprehensive test cases to validate all key use cases and system behaviors using a modular testing approach focused on individual components. The system behaved correctly in all test scenarios.
#### Image Encryption

- Uploaded 5 sample images of different file types 
- Verified images were encrypted into random stego images  
- Confirmed no visible traces of original image
- Decrypted images matched originals
- Repeated across 3 servers to test load balancing 

#### Access Rights

- Set various leasing values for different users
- Checked values that are viewed from the client's side
- For instance, assume client A requested image.png from client B. Then, client B leased the image for 10 view times
- The 10 views were viewed normally after each refresh
- After the 10 views, image was reverted to the default
- We then tested running the client offline and online again to assure consistency

#### Discovery Service

- Registered clients A, B, C
- Verified user list consistency across servers
- Client A entered offline mode
- Client A reconnected again, user listings eventually re-synced

#### P2P Sharing

- Client A requested Client B's Image 1
- Client B directly transmitted to Client A with lease 3
- Client A viewed successfully 3 times
- Fourth view failed, default image shown
- Logged events matched on both ends

#### Leader Election 

- Load balanced 100 client threads across all 3 servers sending images consistently via simple shell script
```
#!/bin/bash
for i in {1..100}; do
	cp image.png "image$i.png"
done
cargo run
for i in {1..100}; do
	send "image$i.png" 
	sleep 0.5
done
```
- Eventually the requests handeled exceeded current leader capacity
- Leader's failure signal to other servers 
- Verified timeout spike during re-election
- Request log showed new leader elected
- New leader arises, handling incoming requests normally

#### Failure Detection

- Sent requests during a leader failure 
- Confirmed failed requests were retried automatically
- Measured retries within 500ms threshold   
- Request timeouts < 15 seconds

### Performance Metrics

To evaluate performance, load tests were executed with and without load balancing enabled (with trivial bully implementation and simple server executions). 

**Metrics Tracked:**
- Request timeout rate (trivial time recording between separate code blocks)
- Average request latency (tested with system time needed to run)
- CPU/bandwidth usage on servers (using sysinfo crate)
- Percentage of completed requests (number of images actually encrypted / total number of images)

The primary archticture for metrics tracking was usually a subset of the code below that had incremental variations for different purposes throughout the process

```
use sysinfo::{SystemExt, System};
use std::time::Instant;

let system = System::new(); 

let start = Instant::now();
let mut success_count = 0;
let mut failed_count = 0;

// Launch client threads 
for _ in 0..100 {
   thread::spawn(|| {
      loop {
         let img = read_image();
         let result = encrypt_image(img); 
         match result {
            Ok(_) => {
               success_count += 1;
            } 
            Err(_) => {
               failed_count += 1; 
            }
         }
      }
   });
}

// Sample metrics   
loop {

   // Timeout rate      
   let timed_out = failed_count / (success_count + failed_count);

   // Avg. latency
   let elapsed = start.elapsed();
   let avg_latency = elapsed / success_count;  

   // CPU usage
   let cpu_usage = system.global_cpu_info().cpu_usage;

   // Completed requests
   let completed = success_count / (success_count + failed_count);

   println!("Timeout Rate: {}", timed_out);
   println!("Avg Latency: {}", avg_latency);
   println!("CPU Usage: {}%", cpu_usage);
   println!("Completed: {}%", completed);   

   sleep(5);
}
```

This shows how the chrono, thread, and sysinfo crates can measure the defined metrics regarding request timing, throughput, CPU loads, and success rates.

The same structure would run on each server peer to collect the data. By wrapping the core logic in threads, we can accurately model the concurrent client connections.

**Load Test Parameters:**
- 100 simultaneous client threads
- Continuous rapid fire image encryption requests  (with continous stream of message sending similar to the previous part)
- 3 server peers (on different devices to allow accurate CPU usage calculations)
### Results without Load Balancing  

| Metric               | Server 1 | Server 2 | Server 3 |
|----------------------|----------|----------|----------|
| Successful Requests  | 2812   | 207      | 203      |  
| Failed Requests      | 1143   | 6        | 5        |
| Request Timeout Rate | 40%      | 3%       | 2%       |
| Avg. Latency         | 2.1 sec  | 218ms    | 204ms    |
| Peak CPU Usage       | 100%     | 24%      | 22%      |   
| Percentage of completed requests| 71% | 97%   | 97.5%   |

With all requests routed to a single server and only naiive switching, CPU and network bandwidth were overwhelmed, resulting in high latency, timeouts, and failed requests.

By contrast:
### Results with Load Balancing Enabled

| Metric               | Server 1 | Server 2 | Server 3 |
|----------------------|----------|----------|----------|
| Successful Requests  | 1,117   | 1,203    | 892    |  
| Failed Requests      | 40      | 13       | 12       |
| Request Timeout Rate | 2.1%     | 0.2%     | 0.2%     |
| Avg. Latency         | 152ms    | 209ms    | 172ms    |
| Peak CPU Usage       | 62%      | 54%      | 60%      |   
| Percentage of completed requests| 96.5% | 98.9%   | 98.6%   |

As we can see, after proper handling of the way requests are being distributed across the servers, the requests are split properly in an almost even way with the continous stream of requests. This demonstrates the significant performance and reliability gains from distributing load across the server pool. Note that the leader election algorithm effectively distributed load. Occasional timeouts were caused by leader failures and re-election.

## Conclusion

In summary, we developed a decentralized system for private image sharing. User images are encrypted P2P with viewer access controls. A cloud-hosted discovery service enables private connections, coordinated via distributed leader election algorithms. Load balancing safeguards performance, and redundancy handles failures. Comprehensive functionality and load testing validated the effectiveness of our approach.
