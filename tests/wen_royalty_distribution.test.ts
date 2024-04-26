import * as anchor from "@coral-xyz/anchor";
import { faker } from "@faker-js/faker";
import { WenNewStandard } from "../target/types/wen_new_standard";
import { TestSale } from "./../target/types/test_sale";
import { WenRoyaltyDistribution } from "./../clients/wns-sdk/src/programs/types/wen_royalty_distribution";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  AccountInfo,
  ComputeBudgetProgram,
} from "@solana/web3.js";

import {
  airdrop,
  getApproveAccountPda,
  getDistributionAccountPda,
  getExtraMetasAccountPda,
  getGroupAccountPda,
  getListingAccountPda,
  getManagerAccountPda,
  getMemberAccountPda,
  getSaleAccountPda,
} from "./utils";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Account,
  TOKEN_2022_PROGRAM_ID,
  getAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { expect } from "chai";

describe("wen_royalty_distribution", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const { connection, wallet } = provider;

  const wnsProgram = anchor.workspace.WenNewStandard as anchor.Program<WenNewStandard>;
  const wenDistributionProgram = anchor.workspace
    .WenRoyaltyDistribution as anchor.Program<WenRoyaltyDistribution>;
  const testSaleProgram = anchor.workspace.TestSale as anchor.Program<TestSale>;

  const wnsProgramId = wnsProgram.programId;
  const wenDistributionProgramId = wenDistributionProgram.programId;
  const testSaleProgramId = testSaleProgram.programId;

  const manager = getManagerAccountPda(wnsProgramId);

  describe("a sale", () => {
    const seller = Keypair.generate();
    const buyer = Keypair.generate();

    const creator1 = Keypair.generate();
    const creator2 = Keypair.generate();

    before(async () => {
      await airdrop(connection, seller.publicKey, 10 * LAMPORTS_PER_SOL);
      await airdrop(connection, buyer.publicKey, 10 * LAMPORTS_PER_SOL);
      await airdrop(connection, creator1.publicKey, 1 * LAMPORTS_PER_SOL);
      await airdrop(connection, creator2.publicKey, 1 * LAMPORTS_PER_SOL);
    });

    describe("using SOL as payment", () => {
      const name = faker.lorem.words({ max: 3, min: 2 });
      const symbol = faker.lorem.word();
      const uri = faker.internet.url();

      const groupMintKeypair = Keypair.generate();
      const memberMintKeypair = Keypair.generate();
      const groupMintPublicKey = groupMintKeypair.publicKey;
      const memberMintPublickey = memberMintKeypair.publicKey;

      const groupMintAuthPublicKey = wallet.publicKey;
      const memberMintAuthPublicKey = seller.publicKey;

      const groupMintTokenAccount = getAssociatedTokenAddressSync(
        groupMintPublicKey,
        groupMintAuthPublicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const sellerMemberMintTokenAccount = getAssociatedTokenAddressSync(
        memberMintPublickey,
        memberMintAuthPublicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const buyerMemberMintTokenAccount = getAssociatedTokenAddressSync(
        memberMintPublickey,
        buyer.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const group = getGroupAccountPda(groupMintPublicKey, wnsProgramId);
      const distribution = getDistributionAccountPda(
        groupMintPublicKey,
        PublicKey.default,
        wenDistributionProgramId
      );
      const sale = getSaleAccountPda(group, distribution, testSaleProgramId);

      const member = getMemberAccountPda(memberMintPublickey, wnsProgramId);
      const extraMetasAccount = getExtraMetasAccountPda(
        memberMintPublickey,
        wnsProgramId
      );
      const approveAccount = getApproveAccountPda(memberMintPublickey, wnsProgramId);

      const listingAmount = new anchor.BN(2 * LAMPORTS_PER_SOL);
      const royaltyBasisPoints = 1000;
      const royalty = listingAmount
        .mul(new anchor.BN(royaltyBasisPoints))
        .div(new anchor.BN(10_000));
      const creator1SharePct = 60;
      const creator2SharePct = 40;

      let distributionAccountInfo: AccountInfo<Buffer>;
      let distributionAccountData;

      before(async () => {
        await testSaleProgram.methods
          .initalizePrepGroup({
            distributionPaymentMint: PublicKey.default,
            group: {
              maxSize: 1,
              name,
              symbol,
              uri,
            },
          })
          .accountsStrict({
            authority: groupMintAuthPublicKey,
            distribution,
            group,
            manager,
            sale,
            mint: groupMintPublicKey,
            mintTokenAccount: groupMintTokenAccount,
            payer: groupMintAuthPublicKey,
            receiver: groupMintAuthPublicKey,
            rent: SYSVAR_RENT_PUBKEY,
            wnsProgram: wnsProgramId,
            distributionProgram: wenDistributionProgramId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([groupMintKeypair])
          .rpc({
            skipPreflight: true,
            preflightCommitment: "confirmed",
            commitment: "confirmed",
          });

        distributionAccountInfo = await connection.getAccountInfo(
          distribution,
          "confirmed"
        );
        distributionAccountData = wenDistributionProgram.coder.accounts.decode(
          "distributionAccount",
          distributionAccountInfo.data
        );
      });

      before(async () => {
        const name = faker.lorem.words({ max: 3, min: 2 });
        const symbol = faker.lorem.word();
        const uri = faker.internet.url();
        await testSaleProgram.methods
          .initalizePrepMint({
            mint: {
              name,
              symbol,
              permanentDelegate: null,
              uri,
            },
            royalties: {
              creators: [
                {
                  address: creator1.publicKey,
                  share: 60,
                },
                { address: creator2.publicKey, share: 40 },
              ],
              royaltyBasisPoints,
            },
          })
          .accountsStrict({
            extraMetasAccount,
            distribution,
            group,
            manager,
            member,
            mintTokenAccount: sellerMemberMintTokenAccount,
            groupUpdateAuthority: groupMintAuthPublicKey,
            authority: memberMintAuthPublicKey,
            mint: memberMintPublickey,
            payer: groupMintAuthPublicKey,
            receiver: memberMintAuthPublicKey,
            rent: SYSVAR_RENT_PUBKEY,
            wnsProgram: wnsProgramId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 })])
          .signers([memberMintKeypair, seller])
          .rpc({
            skipPreflight: true,
            preflightCommitment: "confirmed",
            commitment: "confirmed",
          });
      });

      describe("after initializing distribution", () => {
        it("should have a distribution account", () => {
          expect(distributionAccountInfo).to.not.be.undefined;
        });

        it("should be owned by the distribution program", () => {
          expect(distributionAccountInfo.owner).to.eql(wenDistributionProgramId);
        });

        it("should point the correct group mint account address", () => {
          expect(distributionAccountData.groupMint).to.eql(groupMintPublicKey);
        });

        it("should point the correct payment mint account address", () => {
          expect(distributionAccountData.paymentMint).to.eql(PublicKey.default);
        });
      });

      describe("after listing for sale", () => {
        const listing = getListingAccountPda(
          seller.publicKey,
          memberMintPublickey,
          testSaleProgramId
        );

        let listingAccountInfo: AccountInfo<Buffer>;
        let sellerTokenAccountData: Account;
        let listingAccountData;

        before(async () => {
          await testSaleProgram.methods
            .listNft({
              listingAmount,
              paymentMint: PublicKey.default,
            })
            .accountsStrict({
              listing,
              manager,
              sale,
              payer: wallet.publicKey,
              seller: seller.publicKey,
              mint: memberMintPublickey,
              sellerTokenAccount: sellerMemberMintTokenAccount,
              systemProgram: SystemProgram.programId,
              wnsProgram: wnsProgramId,
              tokenProgram: TOKEN_2022_PROGRAM_ID,
            })
            .signers([seller])
            .rpc({
              skipPreflight: true,
              preflightCommitment: "confirmed",
              commitment: "confirmed",
            });

          listingAccountInfo = await connection.getAccountInfo(listing, "confirmed");
          listingAccountData = testSaleProgram.coder.accounts.decode(
            "listing",
            listingAccountInfo.data
          );
          sellerTokenAccountData = await getAccount(
            connection,
            sellerMemberMintTokenAccount,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
          );
        });

        it("should have a listing account", () => {
          expect(listingAccountData).to.not.be.undefined;
        });

        it("should be owned by sale program", () => {
          expect(listingAccountInfo.owner).to.eql(testSaleProgramId);
        });

        it("should point the sale account as delegate of NFT", () => {
          expect(sellerTokenAccountData.delegate).to.eql(sale);
        });

        it("should freeze the NFT", () => {
          expect(sellerTokenAccountData.isFrozen).to.be.true;
        });
      });

      describe("after a sale", () => {
        const buyerPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          PublicKey.default,
          buyer.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );
        const sellerPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          PublicKey.default,
          seller.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );
        const distributionPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          PublicKey.default,
          distribution,
          true,
          TOKEN_2022_PROGRAM_ID
        );

        const listing = getListingAccountPda(
          seller.publicKey,
          memberMintPublickey,
          testSaleProgramId
        );

        const royalty = listingAmount
          .mul(new anchor.BN(royaltyBasisPoints))
          .div(new anchor.BN(10_000));

        let distributionPreBalance: number;
        let sellerPreBalance: number;
        let buyerPreBalance: number;
        let distributionPostBalance: number;
        let sellerPostBalance: number;
        let buyerPostBalance: number;

        let sellerTokenAccountData: Account;
        let buyerTokenAccountData: Account;

        before(async () => {
          distributionPreBalance = await connection.getBalance(distribution, "confirmed");
          buyerPreBalance = await connection.getBalance(buyer.publicKey, "confirmed");
          sellerPreBalance = await connection.getBalance(seller.publicKey, "confirmed");

          await testSaleProgram.methods
            .fulfillListing({
              buyAmount: listingAmount,
            })
            .accountsStrict({
              approveAccount,
              extraMetasAccount,
              distribution,
              manager,
              listing,
              sale,
              payer: wallet.publicKey,
              buyer: buyer.publicKey,
              seller: seller.publicKey,
              buyerPaymentTokenAccount: buyerPaymentMintTokenAccount,
              sellerPaymentTokenAccount: sellerPaymentMintTokenAccount,
              distributionPaymentTokenAccount: distributionPaymentMintTokenAccount,
              mint: memberMintPublickey,
              paymentMint: PublicKey.default,
              buyerTokenAccount: buyerMemberMintTokenAccount,
              sellerTokenAccount: sellerMemberMintTokenAccount,
              wnsProgram: wnsProgramId,
              distributionProgram: wenDistributionProgramId,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              tokenProgram: TOKEN_2022_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .preInstructions([
              ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
            ])
            .signers([buyer])
            .rpc({
              skipPreflight: true,
              preflightCommitment: "confirmed",
              commitment: "confirmed",
            });

          distributionPostBalance =
            (await connection.getBalance(distribution, "confirmed")) -
            distributionPreBalance;
          buyerPostBalance = await connection.getBalance(buyer.publicKey, "confirmed");
          sellerPostBalance =
            (await connection.getBalance(seller.publicKey, "confirmed")) -
            sellerPreBalance;

          sellerTokenAccountData = await getAccount(
            connection,
            sellerMemberMintTokenAccount,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
          );
          buyerTokenAccountData = await getAccount(
            connection,
            buyerMemberMintTokenAccount,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
          );
        });

        describe("royalties", () => {
          it("should be sent to the distribution vault", () => {
            expect(distributionPostBalance).to.eql(royalty.toNumber());
          });
        });

        describe("the seller", () => {
          it("receive the payment minus royalties", () => {
            expect(sellerPostBalance).to.eql(listingAmount.sub(royalty).toNumber());
          });
          it("should not be the owner anymore", () => {
            expect(sellerTokenAccountData.amount.toString()).to.eql("0");
          });
        });

        describe("the buyer", () => {
          it("sent the payment", () => {
            console.log;
            expect(buyerPostBalance).to.eql(buyerPreBalance - listingAmount.toNumber());
          });
          it("should be the owner", () => {
            expect(buyerTokenAccountData.amount.toString()).to.eql("1");
          });
        });
      });

      describe("after claiming", () => {
        const distributionPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          PublicKey.default,
          distribution,
          true,
          TOKEN_2022_PROGRAM_ID
        );

        describe("creator 1", () => {
          const creator1PaymentMintTokenAccount = getAssociatedTokenAddressSync(
            PublicKey.default,
            creator1.publicKey,
            true,
            TOKEN_2022_PROGRAM_ID
          );

          let creator1PreBalance: number;
          let creator1PostBalance: number;
          const expectedCreatorShare = royalty
            .mul(new anchor.BN(creator1SharePct))
            .div(new anchor.BN(100));

          before(async () => {
            creator1PreBalance = await connection.getBalance(
              creator1.publicKey,
              "confirmed"
            );

            await testSaleProgram.methods
              .claimRoyalty()
              .accountsStrict({
                payer: wallet.publicKey,
                creator: creator1.publicKey,
                distribution,
                paymentMint: PublicKey.default,
                creatorPaymentTokenAccount: creator1PaymentMintTokenAccount,
                distributionPaymentTokenAccount: distributionPaymentMintTokenAccount,
                wenDistributionProgram: wenDistributionProgramId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
              })
              .signers([creator1])
              .rpc({
                skipPreflight: true,
                preflightCommitment: "confirmed",
                commitment: "confirmed",
              });

            creator1PostBalance =
              (await connection.getBalance(creator1.publicKey, "confirmed")) -
              creator1PreBalance;
          });

          it("should receive correct royalty funds", () => {
            expect(creator1PostBalance).to.eql(expectedCreatorShare.toNumber());
          });
        });

        describe("creator 2", () => {
          const creator2PaymentMintTokenAccount = getAssociatedTokenAddressSync(
            PublicKey.default,
            creator2.publicKey,
            true,
            TOKEN_2022_PROGRAM_ID
          );

          let creator2PreBalance: number;
          let creator2PostBalance: number;
          const expectedCreatorShare = royalty
            .mul(new anchor.BN(creator2SharePct))
            .div(new anchor.BN(100));

          before(async () => {
            creator2PreBalance = await connection.getBalance(
              creator2.publicKey,
              "confirmed"
            );

            await testSaleProgram.methods
              .claimRoyalty()
              .accountsStrict({
                payer: wallet.publicKey,
                creator: creator2.publicKey,
                distribution,
                paymentMint: PublicKey.default,
                creatorPaymentTokenAccount: creator2PaymentMintTokenAccount,
                distributionPaymentTokenAccount: distributionPaymentMintTokenAccount,
                wenDistributionProgram: wenDistributionProgramId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
              })
              .signers([creator2])
              .rpc({
                skipPreflight: true,
                preflightCommitment: "confirmed",
                commitment: "confirmed",
              });

            creator2PostBalance =
              (await connection.getBalance(creator2.publicKey, "confirmed")) -
              creator2PreBalance;
          });

          it("should receive correct royalty funds", () => {
            expect(creator2PostBalance).to.eql(expectedCreatorShare.toNumber());
          });
        });
      });
    });

    describe("using SPL token as payment", () => {
      const name = faker.lorem.words({ max: 3, min: 2 });
      const symbol = faker.lorem.word();
      const uri = faker.internet.url();

      const groupMintKeypair = Keypair.generate();
      const memberMintKeypair = Keypair.generate();
      const paymentMintKeypair = Keypair.generate();

      const groupMintPublicKey = groupMintKeypair.publicKey;
      const memberMintPublickey = memberMintKeypair.publicKey;
      const paymentMintPublickey = paymentMintKeypair.publicKey;

      const groupMintAuthPublicKey = wallet.publicKey;
      const memberMintAuthPublicKey = seller.publicKey;
      const paymentMintAuthPublicKey = wallet.publicKey;

      const groupMintTokenAccount = getAssociatedTokenAddressSync(
        groupMintPublicKey,
        groupMintAuthPublicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const sellerMemberMintTokenAccount = getAssociatedTokenAddressSync(
        memberMintPublickey,
        memberMintAuthPublicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const sellerPaymentMintTokenAccount = getAssociatedTokenAddressSync(
        paymentMintPublickey,
        seller.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const buyerMemberMintTokenAccount = getAssociatedTokenAddressSync(
        memberMintPublickey,
        buyer.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const buyerPaymentMintTokenAccount = getAssociatedTokenAddressSync(
        paymentMintPublickey,
        buyer.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const group = getGroupAccountPda(groupMintPublicKey, wnsProgramId);
      const distribution = getDistributionAccountPda(
        groupMintPublicKey,
        paymentMintPublickey,
        wenDistributionProgramId
      );
      const sale = getSaleAccountPda(group, distribution, testSaleProgramId);
      const listing = getListingAccountPda(
        seller.publicKey,
        memberMintPublickey,
        testSaleProgramId
      );
      const member = getMemberAccountPda(memberMintPublickey, wnsProgramId);
      const extraMetasAccount = getExtraMetasAccountPda(
        memberMintPublickey,
        wnsProgramId
      );
      const approveAccount = getApproveAccountPda(memberMintPublickey, wnsProgramId);

      const listingAmount = new anchor.BN(500 * 10 ** 6);
      const royaltyBasisPoints = 1000;
      const royalty = listingAmount
        .mul(new anchor.BN(royaltyBasisPoints))
        .div(new anchor.BN(10_000));
      const creator1SharePct = 60;
      const creator2SharePct = 40;

      let distributionAccountInfo: AccountInfo<Buffer>;
      let distributionAccountData;

      before(async () => {
        await testSaleProgram.methods
          .initalizePrepSpl()
          .accounts({
            authority: paymentMintAuthPublicKey,
            payer: paymentMintAuthPublicKey,
            buyer: buyer.publicKey,
            seller: seller.publicKey,
            mint: paymentMintPublickey,
            buyerTokenAccount: buyerPaymentMintTokenAccount,
            sellerTokenAccount: sellerPaymentMintTokenAccount,
          })
          .signers([paymentMintKeypair])
          .rpc({
            skipPreflight: true,
            preflightCommitment: "confirmed",
            commitment: "confirmed",
          });
      });

      before(async () => {
        await testSaleProgram.methods
          .initalizePrepGroup({
            distributionPaymentMint: paymentMintPublickey,
            group: {
              maxSize: 1,
              name,
              symbol,
              uri,
            },
          })
          .accountsStrict({
            authority: groupMintAuthPublicKey,
            distribution,
            group,
            manager,
            sale,
            mint: groupMintPublicKey,
            mintTokenAccount: groupMintTokenAccount,
            payer: groupMintAuthPublicKey,
            receiver: groupMintAuthPublicKey,
            rent: SYSVAR_RENT_PUBKEY,
            wnsProgram: wnsProgramId,
            distributionProgram: wenDistributionProgramId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .signers([groupMintKeypair])
          .rpc({
            skipPreflight: true,
            preflightCommitment: "confirmed",
            commitment: "confirmed",
          });

        distributionAccountInfo = await connection.getAccountInfo(
          distribution,
          "confirmed"
        );
        distributionAccountData = wenDistributionProgram.coder.accounts.decode(
          "distributionAccount",
          distributionAccountInfo.data
        );
      });

      before(async () => {
        const name = faker.lorem.words({ max: 3, min: 2 });
        const symbol = faker.lorem.word();
        const uri = faker.internet.url();
        await testSaleProgram.methods
          .initalizePrepMint({
            mint: {
              name,
              symbol,
              permanentDelegate: null,
              uri,
            },
            royalties: {
              creators: [
                {
                  address: creator1.publicKey,
                  share: 60,
                },
                { address: creator2.publicKey, share: 40 },
              ],
              royaltyBasisPoints,
            },
          })
          .accountsStrict({
            extraMetasAccount,
            distribution,
            group,
            manager,
            member,
            mintTokenAccount: sellerMemberMintTokenAccount,
            groupUpdateAuthority: groupMintAuthPublicKey,
            authority: memberMintAuthPublicKey,
            mint: memberMintPublickey,
            payer: groupMintAuthPublicKey,
            receiver: memberMintAuthPublicKey,
            rent: SYSVAR_RENT_PUBKEY,
            wnsProgram: wnsProgramId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .preInstructions([ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 })])
          .signers([memberMintKeypair, seller])
          .rpc({
            skipPreflight: true,
            preflightCommitment: "confirmed",
            commitment: "confirmed",
          });
      });

      describe("after initializing distribution", () => {
        it("should have a distribution account", () => {
          expect(distributionAccountInfo).to.not.be.undefined;
        });

        it("should be owned by the distribution program", () => {
          expect(distributionAccountInfo.owner).to.eql(wenDistributionProgramId);
        });

        it("should point the correct group mint account address", () => {
          expect(distributionAccountData.groupMint).to.eql(groupMintPublicKey);
        });

        it("should point the correct payment mint account address", () => {
          expect(distributionAccountData.paymentMint).to.eql(paymentMintPublickey);
        });
      });

      describe("after listing for sale", () => {
        let listingAccountInfo: AccountInfo<Buffer>;
        let sellerTokenAccountData: Account;
        let listingAccountData;

        before(async () => {
          await testSaleProgram.methods
            .listNft({
              listingAmount,
              paymentMint: paymentMintPublickey,
            })
            .accountsStrict({
              listing,
              manager,
              sale,
              payer: wallet.publicKey,
              seller: seller.publicKey,
              mint: memberMintPublickey,
              sellerTokenAccount: sellerMemberMintTokenAccount,
              systemProgram: SystemProgram.programId,
              wnsProgram: wnsProgramId,
              tokenProgram: TOKEN_2022_PROGRAM_ID,
            })
            .signers([seller])
            .rpc({
              skipPreflight: true,
              preflightCommitment: "confirmed",
              commitment: "confirmed",
            });

          listingAccountInfo = await connection.getAccountInfo(listing, "confirmed");
          listingAccountData = testSaleProgram.coder.accounts.decode(
            "listing",
            listingAccountInfo.data
          );
          sellerTokenAccountData = await getAccount(
            connection,
            sellerMemberMintTokenAccount,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
          );
        });

        it("should have a listing account", () => {
          expect(listingAccountData).to.not.be.undefined;
        });

        it("should be owned by sale program", () => {
          expect(listingAccountInfo.owner).to.eql(testSaleProgramId);
        });

        it("should point the sale account as delegate of NFT", () => {
          expect(sellerTokenAccountData.delegate).to.eql(sale);
        });

        it("should freeze the NFT", () => {
          expect(sellerTokenAccountData.isFrozen).to.be.true;
        });
      });

      describe("after a sale", () => {
        const buyerPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          paymentMintPublickey,
          buyer.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );

        const sellerPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          paymentMintPublickey,
          seller.publicKey,
          false,
          TOKEN_2022_PROGRAM_ID
        );

        const distributionPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          paymentMintPublickey,
          distribution,
          true,
          TOKEN_2022_PROGRAM_ID
        );

        let sellerPreBalance: number;
        let buyerPreBalance: number;
        let distributionPostBalance: number;
        let sellerPostBalance: number;
        let buyerPostBalance: number;

        let sellerTokenAccountData: Account;
        let buyerTokenAccountData: Account;

        before(async () => {
          buyerPreBalance = parseInt(
            (
              await getAccount(
                connection,
                buyerPaymentMintTokenAccount,
                "confirmed",
                TOKEN_2022_PROGRAM_ID
              )
            ).amount.toString()
          );
          sellerPreBalance = parseInt(
            (
              await getAccount(
                connection,
                sellerPaymentMintTokenAccount,
                "confirmed",
                TOKEN_2022_PROGRAM_ID
              )
            ).amount.toString()
          );

          await testSaleProgram.methods
            .fulfillListing({
              buyAmount: listingAmount,
            })
            .accountsStrict({
              approveAccount,
              extraMetasAccount,
              distribution,
              manager,
              listing,
              sale,
              payer: wallet.publicKey,
              buyer: buyer.publicKey,
              seller: seller.publicKey,
              buyerPaymentTokenAccount: buyerPaymentMintTokenAccount,
              sellerPaymentTokenAccount: sellerPaymentMintTokenAccount,
              distributionPaymentTokenAccount: distributionPaymentMintTokenAccount,
              mint: memberMintPublickey,
              paymentMint: paymentMintPublickey,
              buyerTokenAccount: buyerMemberMintTokenAccount,
              sellerTokenAccount: sellerMemberMintTokenAccount,
              wnsProgram: wnsProgramId,
              distributionProgram: wenDistributionProgramId,
              associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
              tokenProgram: TOKEN_2022_PROGRAM_ID,
              systemProgram: SystemProgram.programId,
            })
            .preInstructions([
              ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
            ])
            .signers([buyer])
            .rpc({
              skipPreflight: true,
              preflightCommitment: "confirmed",
              commitment: "confirmed",
            });

          distributionPostBalance = parseInt(
            (
              await getAccount(
                connection,
                distributionPaymentMintTokenAccount,
                "confirmed",
                TOKEN_2022_PROGRAM_ID
              )
            ).amount.toString()
          );
          buyerPostBalance = parseInt(
            (
              await getAccount(
                connection,
                buyerPaymentMintTokenAccount,
                "confirmed",
                TOKEN_2022_PROGRAM_ID
              )
            ).amount.toString()
          );
          sellerPostBalance =
            parseInt(
              (
                await getAccount(
                  connection,
                  sellerPaymentMintTokenAccount,
                  "confirmed",
                  TOKEN_2022_PROGRAM_ID
                )
              ).amount.toString()
            ) - sellerPreBalance;

          sellerTokenAccountData = await getAccount(
            connection,
            sellerMemberMintTokenAccount,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
          );
          buyerTokenAccountData = await getAccount(
            connection,
            buyerMemberMintTokenAccount,
            "confirmed",
            TOKEN_2022_PROGRAM_ID
          );
        });

        describe("royalties", () => {
          it("should be sent to the distribution vault", () => {
            expect(distributionPostBalance).to.eql(royalty.toNumber());
          });
        });

        describe("the seller", () => {
          it("receive the payment minus royalties", () => {
            expect(sellerPostBalance).to.eql(listingAmount.sub(royalty).toNumber());
          });
          it("should not be the owner anymore", () => {
            expect(sellerTokenAccountData.amount.toString()).to.eql("0");
          });
        });

        describe("the buyer", () => {
          it("sent the payment", () => {
            console.log;
            expect(buyerPostBalance).to.eql(buyerPreBalance - listingAmount.toNumber());
          });
          it("should be the owner", () => {
            expect(buyerTokenAccountData.amount.toString()).to.eql("1");
          });
        });
      });

      describe("after claiming", () => {
        const distributionPaymentMintTokenAccount = getAssociatedTokenAddressSync(
          paymentMintPublickey,
          distribution,
          true,
          TOKEN_2022_PROGRAM_ID
        );

        describe("creator 1", () => {
          const creator1PaymentMintTokenAccount = getAssociatedTokenAddressSync(
            paymentMintPublickey,
            creator1.publicKey,
            true,
            TOKEN_2022_PROGRAM_ID
          );

          let creator1PostBalance: number;
          const expectedCreatorShare = royalty
            .mul(new anchor.BN(creator1SharePct))
            .div(new anchor.BN(100));

          before(async () => {
            await testSaleProgram.methods
              .claimRoyalty()
              .accountsStrict({
                payer: wallet.publicKey,
                creator: creator1.publicKey,
                distribution,
                paymentMint: paymentMintPublickey,
                creatorPaymentTokenAccount: creator1PaymentMintTokenAccount,
                distributionPaymentTokenAccount: distributionPaymentMintTokenAccount,
                wenDistributionProgram: wenDistributionProgramId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
              })
              .signers([creator1])
              .rpc({
                skipPreflight: true,
                preflightCommitment: "confirmed",
                commitment: "confirmed",
              });

            creator1PostBalance = parseInt(
              (
                await getAccount(
                  connection,
                  creator1PaymentMintTokenAccount,
                  "confirmed",
                  TOKEN_2022_PROGRAM_ID
                )
              ).amount.toString()
            );
          });

          it("should receive correct royalty funds", () => {
            expect(creator1PostBalance).to.eql(expectedCreatorShare.toNumber());
          });
        });

        describe("creator 2", () => {
          const creator2PaymentMintTokenAccount = getAssociatedTokenAddressSync(
            paymentMintPublickey,
            creator2.publicKey,
            true,
            TOKEN_2022_PROGRAM_ID
          );

          let creator2PostBalance: number;
          const expectedCreatorShare = royalty
            .mul(new anchor.BN(creator2SharePct))
            .div(new anchor.BN(100));

          before(async () => {
            await testSaleProgram.methods
              .claimRoyalty()
              .accountsStrict({
                payer: wallet.publicKey,
                creator: creator2.publicKey,
                distribution,
                paymentMint: paymentMintPublickey,
                creatorPaymentTokenAccount: creator2PaymentMintTokenAccount,
                distributionPaymentTokenAccount: distributionPaymentMintTokenAccount,
                wenDistributionProgram: wenDistributionProgramId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                tokenProgram: TOKEN_2022_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
              })
              .signers([creator2])
              .rpc({
                skipPreflight: true,
                preflightCommitment: "confirmed",
                commitment: "confirmed",
              });

            creator2PostBalance = parseInt(
              (
                await getAccount(
                  connection,
                  creator2PaymentMintTokenAccount,
                  "confirmed",
                  TOKEN_2022_PROGRAM_ID
                )
              ).amount.toString()
            );
          });

          it("should receive correct royalty funds", () => {
            expect(creator2PostBalance).to.eql(expectedCreatorShare.toNumber());
          });
        });
      });
    });
  });
});
