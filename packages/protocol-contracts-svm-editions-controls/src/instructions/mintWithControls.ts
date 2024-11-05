import {
  Connection,
  Keypair,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  PublicKey,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import BN from "bn.js";

import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "spl-token-4";
import { getProgramInstanceEditions } from "@rarible_int/protocol-contracts-svm-core";
import { getProgramInstanceEditionsControls } from "@rarible_int/protocol-contracts-svm-core/src/program";
import { getEditionsControlsPda, getHashlistPda, getHashlistMarkerPda, getMinterStatsPda, getMinterStatsPhasePda } from "../utils";
import { IExecutorParams } from "@rarible_int/protocol-contracts-svm-core/src/IExecutorParams";
import { sendSignedTransaction } from "@rarible_int/protocol-contracts-svm-core/src/txUtils";
import { decodeEditions, decodeEditionsControls } from "@rarible_int/protocol-contracts-svm-core/src/program";
import { PROGRAM_ID_GROUP_EXTENSIONS } from "@rarible_int/protocol-contracts-svm-core/src/program";
import { IMintWithControls } from "../model";


const MAX_MINTS_PER_TRANSACTION = 3;

export const mintWithControls = async ({
  wallet,
  params,
  connection,
}: IExecutorParams<IMintWithControls>) => {
  const {
    phaseIndex,
    editionsId,
    numberOfMints,
    merkleProof,
    allowListPrice,
    allowListMaxClaims,
    isAllowListMint,
  } = params;

  const editionsControlsProgram = getProgramInstanceEditionsControls(connection);

  const editions = new PublicKey(editionsId);

  const editionsData = await connection.getAccountInfo(editions);

  if (!editionsData) {
    throw Error("Editions not found");
  }

  const raribleEditionsProgram = getProgramInstanceEditions(connection);

  const editionsObj = decodeEditions(raribleEditionsProgram)(editionsData.data, editions);

  const editionsControlsPda = getEditionsControlsPda(editions);

  const editionsControlsData = await connection.getAccountInfo(editionsControlsPda);

  const editionsControlsObj = decodeEditionsControls(editionsControlsProgram)(
    editionsControlsData.data,
    editionsControlsPda
  );

  const hashlist = getHashlistPda(editions)[0];

  const minterStats = getMinterStatsPda(editions, wallet.publicKey)[0];

  const minterStatsPhase = getMinterStatsPhasePda(editions, wallet.publicKey, phaseIndex)[0];

  let remainingMints = numberOfMints;

  let txs: Transaction[] = [];

  while (remainingMints > 0) {
    const instructions: TransactionInstruction[] = [];
    /// creates an open editions launch

    instructions.push(
      ComputeBudgetProgram.setComputeUnitLimit({
        units: 850_000,
      })
    );

    const mints: Keypair[] = [];
    const members: Keypair[] = [];

    for (let i = 0; i < Math.min(MAX_MINTS_PER_TRANSACTION, remainingMints); ++i) {
      const mint = Keypair.generate();
      const member = Keypair.generate();

      mints.push(mint);
      members.push(member);

      const tokenAccount = getAssociatedTokenAddressSync(
        mint.publicKey,
        wallet.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const hashlistMarker = getHashlistMarkerPda(editions, mint.publicKey)[0];

      instructions.push(
        await editionsControlsProgram.methods
          .mintWithControls({
            phaseIndex,
            merkleProof: isAllowListMint ? merkleProof : null,
            allowListPrice: isAllowListMint ? new BN(allowListPrice) : null,
            allowListMaxClaims: isAllowListMint ? new BN(allowListMaxClaims) : null,
          })
          .accountsStrict({
            editionsDeployment: editions,
            editionsControls: editionsControlsPda,
            hashlist,
            hashlistMarker,
            payer: wallet.publicKey,
            mint: mint.publicKey,
            member: member.publicKey,
            signer: wallet.publicKey,
            minter: wallet.publicKey,
            minterStats,
            minterStatsPhase,
            group: editionsObj.item.group,
            groupMint: editionsObj.item.groupMint,
            platformFeeRecipient1: editionsControlsObj.item.platformFeeRecipients[0].address,
            groupExtensionProgram: PROGRAM_ID_GROUP_EXTENSIONS,
            tokenAccount,
            treasury: editionsControlsObj.item.treasury,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            raribleEditionsProgram: raribleEditionsProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            
          })
          .signers([mint, member])
          .instruction()
      );
    }

    remainingMints -= MAX_MINTS_PER_TRANSACTION;

    // transaction boilerplate - ignore for now
    const tx = new Transaction().add(...instructions);
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    tx.feePayer = wallet.publicKey;
    tx.sign(...mints, ...members);
    txs.push(tx);
  }

  await wallet.signAllTransactions(txs);

  const promises = txs.map(item =>
    sendSignedTransaction({
      signedTransaction: item,
      connection,
      skipPreflight: false,
    })
  );

  await Promise.all(promises);

  console.log("Minting successful.");

  return { editions, editionsControls: editionsControlsPda };
};
