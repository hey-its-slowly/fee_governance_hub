import * as anchor from "@coral-xyz/anchor";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { FeeGovernanceHub } from "../idl/fee_governance_hub";
import * as FeeGovernanceHubIdl from "../idl/fee_governance_hub.json";
import { PublicKey } from "@solana/web3.js";

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

export const PROGRAM_ID = new PublicKey(FeeGovernanceHubIdl.address);

export const MAX_FEE_WALLETS_LEN = 3;
export const MAX_FEE_INSTRUCTION_NAME_LEN = 30;

export const PERCENT_DENOMINATOR = 1000;

export const GLOBAL_FEE_WALLETS = [
  {
    address: new PublicKey("ArpaDqpkJpKfxLP7WoFvYMbkj33C1PAHcy8tyrxFpgrc"),
    feePercent: 1000,
  },
];
