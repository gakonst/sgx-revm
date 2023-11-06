# SGX - REVM

**PoC illustrating usage of the [Fortanix SGX platform](https://edp.fortanix.com/docs/) for executing
an EVM message confidentially without leaking any compute or access
information.**

## How it works

0. User manually provisions an [SGX server, e.g. on Azure](https://learn.microsoft.com/en-us/azure/confidential-computing/quick-create-portal)
1. SGX-enabled server opens up a TCP Socket with TLS Enabled (assumes some kind of Certificate is already generated, see first line in main.rs - ideally there's a productionized way to do Certificate provisioning).
2. User submits a TLS-encrypted payload to the server, ensuring the user and the server only have access to the information being delivered (the server actually doesn't because the socket is opened within the SGX enclave).
3. The Server proceeds to parse the payload into an EVM message and execute it _confidentially_.

The EVM database is expected to be instantiated as _empty_, and the user is expected to provide a payload which contains all the storage slots & values required by their transaction, _including Merkle Patricia Proofs_ for proving that these transactions are part of the actual state. It assumes that there is also a state root available to check against.

## TODO

1. Make the demo unit-testable for CI usage
1. Enable TLS payload decryption on the server (currently we just submit plaintext payloads to make prototype testing with netcat easier)
1. Extend the user-submitted payload to multiple transactions including merkle patricia proof verification for each access. This is kind of like a stateless node / light-client.
1. [Remote Attestation](https://edp.fortanix.com/docs/examples/attestation/)

## How to replicate the results

### Installing the Fortanix SDK Platform

On an SGX-supported machine (e.g. on Azure), you'll need to install the [Fortanix SDK](https://edp.fortanix.com/docs/).
Providing a quick-start below:

```bash
# From: https://edp.fortanix.com/docs/installation/guide/
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Nightly -- required for SGX below
rustup toolchain install nightly

# Add SGX the platform
rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly

# Missing some things..
sudo apt-get install -y pkg-config libssl-dev protobuf-compiler cmake clang

# Install the CLI tools
cargo install fortanix-sgx-tools sgxs-tools

# Override the default cargo runner with the SGX one
echo >> ~/.cargo/config -e '[target.x86_64-fortanix-unknown-sgx]\nrunner = "ftxsgx-runner-cargo"'

# Install DKMS & the SGX Service
echo "deb https://download.fortanix.com/linux/apt xenial main" | sudo tee -a /etc/apt/sources.list.d/fortanix.list >/dev/null
echo "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list >/dev/null

curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | sudo -E apt-key add -
curl -sSL "https://download.fortanix.com/linux/apt/fortanix.gpg" | sudo -E apt-key add -

sudo apt-get update
sudo apt-get install intel-sgx-dkms sgx-aesm-service libsgx-aesm-launch-plugin

# Check your SGX setup, all should be green except the `libsgx_enclave_common` maybe.
sgx-detect
```

### Running the demo

On a terminal run:
```
cargo run --release --target x86_64-fortanix-unknown-sgx
```

On another terminal:
```
nc localhost 7878
{ "sender": "0xdafea492d9c6733ae3d56b7ed1adb60692c98bc5", "amount": 40 }
```

Then CTRL+C to close the netcat session, and you'll see on the first terminal that the simulation has completed, without the host ever knowing what happened!
