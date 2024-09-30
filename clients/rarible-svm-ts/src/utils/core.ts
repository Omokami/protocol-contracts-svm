import {
	AnchorProvider, type Idl, type Instruction, Program, type Provider,
	utils,
} from '@coral-xyz/anchor';
import {
	type AccountMeta, Connection, PublicKey, type TransactionInstruction,
} from '@solana/web3.js';
import {
	marketplaceProgramId,
	metadataSeed,
	metaplexMetadataProgramId,
	wnsDistributionProgramId,
	wnsProgramId,
} from './constants';
import {
	ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccount, createAssociatedTokenAccountInstruction, getExtraAccountMetaAddress, getExtraAccountMetas, getMint, getTokenMetadata, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import {type Marketplace, marketplaceIdl} from '../program';

export const getProvider = () => {
	const connection = new Connection(process.env.RPC_URL ?? 'https://api.devnet.solana.com');
	const anchorProvider = AnchorProvider.local();
	const provider = new AnchorProvider(connection, anchorProvider.wallet, {...AnchorProvider.defaultOptions(), commitment: 'processed'});
	return provider;
};

export const getTokenProgramFromMint = async (provider: Provider, mint: string) => {
	const mintPubkey = new PublicKey(mint);
	try {
		await getMint(provider.connection, mintPubkey, undefined, TOKEN_PROGRAM_ID);
		return TOKEN_PROGRAM_ID;
	} catch (e) {
		try {
			await getMint(provider.connection, mintPubkey, undefined, TOKEN_2022_PROGRAM_ID);
			return TOKEN_2022_PROGRAM_ID;
		} catch (e) {
			return undefined;
		}
	}
};

export const getNftProgramFromMint = async (provider: Provider, nftMint: string) => {
	const mintProgram = await getTokenProgramFromMint(provider, nftMint);
	if (!mintProgram) {
		return undefined;
	}

	if (mintProgram === TOKEN_PROGRAM_ID) {
		const isMetaplex = await isMetaplexMetadataAccount(provider, nftMint);
		if (isMetaplex) {
			return metaplexMetadataProgramId;
		}
	}

	if (mintProgram === TOKEN_2022_PROGRAM_ID) {
		const isWns = await isWnsNft(provider, nftMint);
		if (isWns) {
			return wnsProgramId;
		}
	}

	return undefined;
};

export const isMetaplexMetadataAccount = async (provider: Provider, mint: string) => {
	const mintPubkey = new PublicKey(mint);
	const metadataAccount = getProgramAddress(
		[Buffer.from(metadataSeed), metaplexMetadataProgramId.toBytes(), mintPubkey.toBytes()],
		metaplexMetadataProgramId,
	);

	try {
		await provider.connection.getAccountInfo(metadataAccount);
		return true;
	} catch (e) {
		return false;
	}
};

export const isWnsNft = async (provider: Provider, mint: string) => {
	const mintPubkey = new PublicKey(mint);
	const metadata = await getTokenMetadata(provider.connection, mintPubkey);

	if (metadata === null) {
		return false;
	}

	const extraMeta = metadata.additionalMetadata;
	const royalties = extraMeta.filter(m => m[0] === 'royalty_basis_points');
	if (royalties.length > 0) {
		return true;
	}

	return false;
};

export type WnsAccountParams = {
	groupMint: string;
	paymentMint: string;
};

export const getRemainingAccountsForMint = async (provider: Provider, mint: string, wnsParams: WnsAccountParams | undefined) => {
	const remainingAccounts: AccountMeta[] = [];

	const nftProgram = await getNftProgramFromMint(provider, mint);

	if (nftProgram === wnsProgramId) {
		if (!wnsParams) {
			return [];
		}

		const extraMetaPda = getExtraMetasAccountPda(mint);
		const approveAccount = getApproveAccountPda(mint);
		const distributionAccount = getDistributionAccountPda(wnsParams.groupMint, wnsParams.paymentMint);

		remainingAccounts.push(...[
			{
				pubkey: approveAccount,
				isWritable: true,
				isSigner: false,
			},
			{
				pubkey: distributionAccount,
				isWritable: true,
				isSigner: false,
			},
			{
				pubkey: PublicKey.default,
				isWritable: false,
				isSigner: false,
			},
			{
				pubkey: wnsDistributionProgramId,
				isWritable: false,
				isSigner: false,
			},
			{
				pubkey: extraMetaPda,
				isWritable: false,
				isSigner: false,
			},
			{
				pubkey: approveAccount,
				isWritable: false,
				isSigner: false,
			},
			{
				pubkey: wnsProgramId,
				isWritable: false,
				isSigner: false,
			},
		]);
		return remainingAccounts;
	}

	// Need todo metaplex pNFT accounts
	return [];
};

export const getMarketplaceProgram = (provider: Provider) => new Program(
	marketplaceIdl as Idl,
	provider,
) as unknown as Program<Marketplace>;

export const getProgramAddress = (seeds: Uint8Array[], programId: PublicKey) => {
	const [key] = PublicKey.findProgramAddressSync(seeds, programId);
	return key;
};

export const getAtaAddress = (mint: string, owner: string, tokenProgram: string): PublicKey => getProgramAddress(
	[new PublicKey(owner).toBuffer(), new PublicKey(tokenProgram).toBuffer(), new PublicKey(mint).toBuffer()],
	ASSOCIATED_TOKEN_PROGRAM_ID,
);

// MARKET ACCOUNTS
export const getMarketPda = (marketIdentifier: string) => {
	const [marketAccount] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('market'), new PublicKey(marketIdentifier).toBuffer()], marketplaceProgramId);

	return marketAccount;
};

export const getVerificationPda = (marketAddress: string, nftMint: string) => {
	const [marketAccount] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('verification'), new PublicKey(nftMint).toBuffer(), new PublicKey(marketAddress).toBuffer()], marketplaceProgramId);

	return marketAccount;
};

export const getOrderAccount = (nonce: string, marketAddress: string, user: string) => {
	const [marketAccount] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('order'), new PublicKey(nonce).toBuffer(), new PublicKey(marketAddress).toBuffer(), new PublicKey(user).toBuffer()], marketplaceProgramId);

	return marketAccount;
};

export const getEventAuthority = () => {
	const [eventAuthority] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('__event_authority')], marketplaceProgramId);

	return eventAuthority;
};

// WNS ACCOUNTS
export const getManagerAccountPda = () => {
	const [managerAccount] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('manager')], wnsProgramId);

	return managerAccount;
};

export const getExtraMetasAccountPda = (mint: string) => {
	const [extraMetasAccount] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('extra-account-metas'), new PublicKey(mint).toBuffer()], wnsProgramId);

	return extraMetasAccount;
};

export const getApproveAccountPda = (mint: string) => {
	const [approveAccount] = PublicKey.findProgramAddressSync([utils.bytes.utf8.encode('approve-account'), new PublicKey(mint).toBuffer()], wnsProgramId);

	return approveAccount;
};

export const getDistributionAccountPda = (groupMint: string, paymentMint: string) => {
	const [distributionAccount] = PublicKey.findProgramAddressSync([new PublicKey(groupMint).toBuffer(), new PublicKey(paymentMint).toBuffer()], wnsDistributionProgramId);

	return distributionAccount;
};

