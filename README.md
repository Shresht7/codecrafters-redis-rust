[![progress-banner](https://backend.codecrafters.io/progress/redis/b3aac796-d508-4680-85c9-09ef0040b12b)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a starting point for Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

In this challenge, you'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

>> [!NOTE]
> To test the server locally, use the `redis-cli` client application.

---

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

---

## Stage 2: Respond to PING

In this stage, we implement a Redis server that responds to the [`PING`](https://redis.io/commands/ping) command.

Redis clients communicate with Redis servers by sending "[commands](https://redis.io/docs/latest/commands/)", and the server responds with a reply. Both, "commands" and "responses" are encoded using the [Redis Protocol][Redis Protocol]

`PING` is one of the simplest Redis commands. It's used to check whether a Redis server is healthy. The server responds with a `PONG\r\n` message.

> [!NOTE]
> The exact bytes your program receives won't just be `PING`, it'll be something like `*1\r\n$4\r\nPING\r\n`. This is because Redis commands are encoded using the [Redis Protocol][Redis Protocol]. This is handled in later stages.

### 📕 References

- 📄 [Redis: PING Command](https://redis.io/commands/ping)
- 📄 https://lethain.com/redis-protocol/
- 📄 [Redis Protocol][Redis Protocol]
- 📄 [Rust TCP Server](https://app.codecrafters.io/concepts/rust-tcp-server)

---

## Stage 3: Respond to multiple PINGs

In this stage, we respond to multiple PING commands sent by the **same connection**.

A Redis server starts to listen for the next command as soon as it's done responding to the previous one, in the same connection. This allows Redis clients to send multiple commands in quick succession.

---

## Stage 4: Handle concurrent clients

In this stage, we add support for multiple concurrent clients.

In addition to handling multiple commands from the same connection, Redis servers are also designed to handle multiple clients at once. This can be done by using threads, or (like Redis) by using a single-threaded event loop.

### 📕 References

- 📄 https://rohitpaulk.com/articles/redis-3
- 📽️ [YouTube: Phillip Robert - What the heck is the event loop anyway?](https://www.youtube.com/watch?v=8aGhZQkoFbQ&ab_channel=JSConf)
- 📽️ [YouTube: JSConf - Event Loop](https://www.youtube.com/watch?v=cCOL7MC4Pl0)

---

## Stage 5: Implement the ECHO command

In this stage, we add support for the [`ECHO`](https://redis.io/commands/echo) command.

`ECHO`, like `PING`, is a command mostly used for testing and debugging. It simply returns the message that was sent to it.

```sh
$ redis-cli PING # The command we implemented in the previous stages
PONG
$ redis-cli ECHO "Hello, World!"
"Hello, World!"
```

> [!NOTE]
> Redis commands are case-insensitive, so `ECHO`, `echo`, and `EcHo` are all valid commands.

### 📕 References

- [RESP Bulk String](https://redis.io/docs/reference/protocol-spec/#bulk-strings)
- [RESP: Redis Serialization Protocol][Redis Protocol]

---

<!-- ----- -->
<!-- LINKS -->
<!-- ----- -->

[TCP]: https://en.wikipedia.org/wiki/Transmission_Control_Protocol
[Redis Protocol]: https://redis.io/topics/protocol
