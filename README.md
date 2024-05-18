[![progress-banner](https://backend.codecrafters.io/progress/redis/b3aac796-d508-4680-85c9-09ef0040b12b)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

In this challenge, you'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

## Stage 1: Bind to a Port

In this stage, we implement a TCP server that listens on port `6379`.

[TCP][TCP] is the underlying protocol used by protocols like HTTP, HTTPS, SSH and more. Redis server and clients use TCP to communicate with each other.

> [!NOTE]
> Redis uses port `6379`. If you already have a Redis server running on your machine and listening on port `6379`, you'll see a "port already in use" error. You can either stop the Redis server or change the port number in your code.

### 📕 Reference

- 📄 [Wikipedia: TCP][TCP]
- 📽️ [YouTube (Julia Evans): fun with Sockets: let's write a web-server!](https://www.youtube.com/watch?v=1HF-UAGcuvs)
- 📽️ [YouTube (BenEater): Networking Tutorial Playlist](https://www.youtube.com/playlist?list=PLowKtXNTBypH19whXTVoG3oKSuOcw_XeW)
- 📄 [Network Protocols](https://app.codecrafters.io/concepts/network-protocols)
- 📄 [TCP: An Overview](https://app.codecrafters.io/concepts/network-protocols)



<!-- ----- -->
<!-- LINKS -->
<!-- ----- -->

[TCP]: https://en.wikipedia.org/wiki/Transmission_Control_Protocol
