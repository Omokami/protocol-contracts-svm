import { ethers } from "ethers";
import dotenv from "dotenv";
import { RawEntry, CsvEntry, RejectedEntry } from "./utils";
import { PublicKey } from "@solana/web3.js";

dotenv.config();

export async function verifyBurnTransaction(
  provider: ethers.JsonRpcProvider,
  txHash: string,
  expectedSigner: string,
  expectedContract: string,
  message: string,
  signature: string,
  address: string // minter address, recipient of minting rights
): Promise<boolean> {
  const prefix = process.env.CHAIN_NAME;
  const deprefixedSigner = expectedSigner.replace(new RegExp(`^${prefix}:`), "");

  // Verify the signature
  const messageHash = ethers.hashMessage(message);
  const recoveredAddress = ethers.recoverAddress(messageHash, signature);

  if (recoveredAddress.toLowerCase() !== deprefixedSigner.toLowerCase()) {
    throw new Error("!SignatureMatch");
  }

  // Fetch the transaction
  const tx = await provider.getTransaction(txHash);

  // Check if the transaction is to the expected contract
  if (tx.to.toLowerCase() !== expectedContract.toLowerCase()) {
    throw new Error("!ContractAddressMatch");
  }

  // Lastly, verify that the address is valid.
  // should be a valid Solana address
  try {
    const isValidSolanaAddress = PublicKey.isOnCurve(address);
    if (!isValidSolanaAddress) {
      throw new Error("!ValidMinterAddress");
    }
  } catch (error) {
    throw new Error("!ValidMinterAddress");
  }

  return true;
}

export async function verifyTransactionBatch(
  rawEntries: RawEntry[],
  CONTRACT_ADDRESS: string,
  batchSize: number = 1000
) {
  const provider = new ethers.InfuraProvider(process.env.NETWORK, process.env.INFURA_PROJECT_ID);

  const verifiedBurners: CsvEntry[] = [];
  const rejectedBurners: RejectedEntry[] = [];

  // Process in batches
  for (let i = 0; i < rawEntries.length; i += batchSize) {
    const batch = rawEntries.slice(i, i + batchSize);
    console.log(
      `Processing batch ${i / batchSize + 1} of ${Math.ceil(rawEntries.length / batchSize)}`
    );

    const verificationPromises = batch.map(async entry => {
      try {
        const isVerified = await verifyBurnTransaction(
          provider,
          entry.tx_hash,
          entry.signer,
          CONTRACT_ADDRESS,
          entry.message,
          entry.signature,
          entry.address
        );
        if (isVerified) {
          verifiedBurners.push({ address: entry.address, quantity: entry.quantity });
        }
      } catch (error) {
        rejectedBurners.push({
          address: entry.address,
          quantity: entry.quantity,
          rejectionReason: error.message,
        });
      }
    });

    await Promise.all(verificationPromises);
    console.log(`Completed batch ${i / batchSize + 1}\n`);
  }

  return { verifiedBurners, rejectedBurners };
}
