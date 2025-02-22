import {
  AutocratClient,
  CreateClientParams,
  PriceMath,
} from "@metadaoproject/futarchy/dist/v0.4";
import {
  createMint,
  getAssociatedTokenAddressSync,
  mintTo,
} from "@solana/spl-token";
import { Keypair } from "@solana/web3.js";
import BN from "bn.js";

export async function createProposal({
  payer,
  clientParams,
  amountToken = 1000000000000000,
  amountUsdc = 1000000000000000,
}: {
  payer: Keypair;
  clientParams: CreateClientParams;
  amountToken?: bigint | number;
  amountUsdc?: bigint | number;
}) {
  const client = AutocratClient.createClient(clientParams);

  const meta = await createMint(
    client.autocrat.provider.connection,
    payer,
    payer.publicKey,
    null,
    6
  );
  const usdc = await createMint(
    client.autocrat.provider.connection,
    payer,
    payer.publicKey,
    null,
    6
  );

  const metaAccount = getAssociatedTokenAddressSync(
    meta,
    payer.publicKey,
    true
  );
  await mintTo(
    client.autocrat.provider.connection,
    payer,
    meta,
    metaAccount,
    payer,
    amountToken
  );
  const usdcAccount = getAssociatedTokenAddressSync(
    usdc,
    payer.publicKey,
    true
  );
  await mintTo(
    client.autocrat.provider.connection,
    payer,
    usdc,
    usdcAccount,
    payer,
    amountUsdc
  );

  const dao = await client.initializeDao(meta, 1000, 100, 100, usdc);

  const accounts = [
    {
      pubkey: dao,
      isSigner: true,
      isWritable: true,
    },
  ];
  const data = client.autocrat.coder.instruction.encode("update_dao", {
    daoParams: {
      passThresholdBps: 500,
      baseBurnLamports: null,
      burnDecayPerSlotLamports: null,
      slotsPerProposal: null,
      marketTakerFee: null,
    },
  });
  const instruction = {
    programId: client.autocrat.programId,
    accounts,
    data,
  };
  const proposal = await client.initializeProposal(
    dao,
    "",
    instruction,
    PriceMath.getChainAmount(5, 9),
    PriceMath.getChainAmount(5000, 6)
  );
  const proposalPdas = client.getProposalPdas(proposal, meta, usdc, dao);

  return {
    client,
    dao,
    proposal,
    meta,
    usdc,
    metaAccount,
    usdcAccount,
    proposalPdas,
  };
}
