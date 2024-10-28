import {
	PublicKey,
	SystemProgram,
} from '@solana/web3.js';
import {type Provider, BN} from '@coral-xyz/anchor';
import {
	getEventAuthority, getMarketPda, getMarketplaceProgram,
	getVerificationPda,
	marketplaceProgramId,
} from '../utils';

export type InitMarketParams = {
	marketIdentifier: string;
	feeRecipient: string;
	feeBps: number;
};

// Initialize Market
export const getInitializeMarket = async (provider: Provider, marketParams: InitMarketParams) => {
	const marketProgram = getMarketplaceProgram(provider);
	const market = getMarketPda(marketParams.marketIdentifier);
	const eventAuthority = getEventAuthority();

	const ix = await marketProgram.methods
		.initMarket({ feeBps: new BN(marketParams.feeBps), feeRecipient: new PublicKey(marketParams.feeRecipient) })
		.accountsStrict({
			initializer: provider.publicKey,
			marketIdentifier: marketParams.marketIdentifier,
			market,
			systemProgram: SystemProgram.programId,
			program: marketplaceProgramId,
			eventAuthority,
		})
		.instruction();
	return ix;
};

// Verify Mint
export const getVerifyMint = async (provider: Provider, nftMint: string, marketAddress: string) => {
	const marketProgram = getMarketplaceProgram(provider);
	const verificationPda = getVerificationPda(marketAddress, nftMint);

	const ix = await marketProgram.methods
		.verifyMint()
		.accountsStrict({
			initializer: provider.publicKey,
			market: marketAddress,
			nftMint,
			verification: verificationPda,
			systemProgram: SystemProgram.programId,
		})
		.instruction();
	return ix;
};
