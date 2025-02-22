import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Lobbyist } from "../target/types/lobbyist";
import { createProposal } from "./helpers";

describe("lobbyist", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Lobbyist as Program<Lobbyist>;

  it("Lobby", async () => {
    const payer = anchor.web3.Keypair.generate();
    await provider.connection.requestAirdrop(
      payer.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );

    const {
      client,
      dao,
      proposal,
      meta,
      usdc,
      metaAccount,
      usdcAccount,
      proposalPdas,
    } = await createProposal({
      payer: payer,
      clientParams: {
        provider: provider,
      },
    });
    await program.methods
      .initializeLobbyist()
      .accounts({
        lobbyist: program.programId,
        creator: payer.publicKey,
        proposal: proposal,
        tokenMint: metaAccount,
        usdcMint: usdcAccount,
      })
      .rpc();
    await program.methods
      .initializeEscrow()
      .accounts({
        lobbyist: program.programId,
        creator: payer.publicKey,
        proposal: proposal,
        tokenMint: metaAccount,
        usdcMint: usdcAccount,
      })
      .rpc();
  });
});
