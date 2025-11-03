## Lobbyist Program

A Solana program that automates trading [MetaDAO](https://metadao.fi/)'s decision markets based on user preferences.

### High-level Architecture

Token holders of a given futarchy can deposit tokens into an escrow they created, providing their relative preferences. Crankers can then come in and permissionlessly trigger trades from the escrow, earn a small fee doing so.

```mermaid
flowchart TB
    subgraph Futarchy
        subgraph Markets
            Spot
            Pass
            Fail
        end
        DAO
        Proposal
    end

    subgraph Lobbyist
        Es["Escrow"]
    end

    subgraph Users
        Th["Token Holder"]
        C["Crankers"]
    end

    Es --. Derived from .--> Proposal
    Proposal --. Derived from .--> DAO
    Th --. Deposits tokens .--> Es
    C --. Triggers trade .--> Es
    Es --. Trades .--> Spot
    Es --. Trades .--> Pass
    Es --. Trades .--> Fail
    DAO --. Derived from .--> Spot
    DAO --. Derived from .--> Pass
    DAO --. Derived from .--> Fail
```

### Development

Prereqs: Rust + Solana toolchain and dependencies already configured for this repo.

Build the program:

```bash
cargo build-sbf
```

Run tests:

```bash
cargo test -- --nocapture
```
