import {
	SystemProgram,
} from '@solana/web3.js';
import {type Provider} from '@coral-xyz/anchor';
import {
	getEventAuthority, getMarketPda, getMarketplaceProgram,
	getVerificationPda,
	marketplaceProgramId,
} from '../utils';

// Initialize Market
export const getInitializeMarket = async (provider: Provider, marketIdentifier: string) => {
	const marketProgram = getMarketplaceProgram(provider);
	const market = getMarketPda(marketIdentifier);
	const eventAuthority = getEventAuthority();

	const ix = await marketProgram.methods
		.initMarket()
		.accountsStrict({
			initializer: provider.publicKey,
			marketIdentifier,
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
