import * as anchor from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { FeeGovernanceHub } from "../idl/fee_governance_hub";
import * as FeeGovernanceHubIdl from "../idl/fee_governance_hub.json";

// constants for seeds
export const CONFIG_TAG = Buffer.from("CONFIG_TAG");

export const getFeeGovernanceHubProgram = (
  connection: anchor.web3.Connection = new anchor.web3.Connection(
    "https://api.devnet.solana.com"
  )
) => {
  const keypair = anchor.web3.Keypair.generate();
  if (!keypair) return null;
  const provider = new anchor.AnchorProvider(
    connection,
    new NodeWallet(keypair),
    anchor.AnchorProvider.defaultOptions()
  );
  const program = new anchor.Program(
    FeeGovernanceHubIdl as unknown as FeeGovernanceHub,
    provider
  );

  return program;
};
