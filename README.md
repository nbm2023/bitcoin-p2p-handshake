# bitcoin-p2p-handshake

Simple implementation of the Bitcoin p2p handhshake protocol, according to https://en.bitcoin.it/wiki/Protocol_documentation

We advertie our version to a Bitcoin node with by sending a so called `VersionMessage` and wait for the remote node to acknowledge it with a `VerackMessage` and its own `VersionMessage`. At that point we acknowledge the node's version message with our own `VerackMessage`, and that concludes the handshake protocol.

This implementation can be tested by running the `test_handshake` unit test with `cargo t -- --nocapture`
