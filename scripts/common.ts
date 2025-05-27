import * as anchor from "@coral-xyz/anchor";
const solanaNetworkRpc = process.env.ANCHOR_PROVIDER_URL;

export const getConnection = () => {
  console.log("solanaNetworkRpc", solanaNetworkRpc);
  return new anchor.web3.Connection(solanaNetworkRpc, "confirmed");
};

export const getAdminKeypair = () => {
  const admin_secret_key = process.env.ADMIN_SECRET_KEY;
  if (!admin_secret_key) return null;

  const adminKeypair = anchor.web3.Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(admin_secret_key))
  );

  console.log("key", adminKeypair.publicKey.toBase58());

  return adminKeypair;
};
