/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {type Provider, BN} from '@coral-xyz/anchor';
import {ASSOCIATED_TOKEN_PROGRAM_ID} from '@solana/spl-token';
import {
	SYSVAR_INSTRUCTIONS_PUBKEY, SystemProgram, PublicKey,
	Keypair,
	type AccountMeta,
} from '@solana/web3.js';
import {
	getMarketplaceProgram, getMarketPda, getOrderAccount, getVerificationPda, getEventAuthority, marketplaceProgramId, fetchOrderByAddress, getTokenProgramFromMint, getNftProgramFromMint, getAtaAddress,
	getRemainingAccountsForMint,
	type WnsAccountParams,
} from '../utils';

export type ListArgs = {
	marketIdentifier: string;
	nftMint: string;
	paymentMint: string;
	price: number;
	extraAccountParams: WnsAccountParams | undefined; // Add metaplex
};
// List NFT
export const getListNft = async (provider: Provider, listingArgs: ListArgs) => {
	const marketProgram = getMarketplaceProgram(provider);
	const market = getMarketPda(listingArgs.marketIdentifier);
	const initializer = provider.publicKey?.toString();
	if (!initializer) {
		return undefined;
	}

	const nftMint = listingArgs.nftMint.toString();
	const nftTokenProgram = await getTokenProgramFromMint(provider, listingArgs.nftMint.toString());
	if (!nftTokenProgram) {
		return undefined;
	}

	const nonceKp = Keypair.generate();
	const nonce = nonceKp.publicKey;

	const nftProgram = await getNftProgramFromMint(provider, nftMint);

	const order = getOrderAccount(nonce.toString(), market.toString(), initializer);
	const initializerNftTa = getAtaAddress(listingArgs.nftMint, initializer, nftTokenProgram.toString());
	const orderNftTa = getAtaAddress(listingArgs.nftMint, order.toString(), nftTokenProgram.toString());

	const verification = getVerificationPda(market.toString(), listingArgs.nftMint);
	const eventAuthority = getEventAuthority();

	const remainingAccounts: AccountMeta[] = await getRemainingAccountsForMint(provider, nftMint, listingArgs.extraAccountParams);

	const ix = await marketProgram.methods
		.list({
			nonce,
			paymentMint: new PublicKey(listingArgs.paymentMint),
			price: new BN(listingArgs.price),
		})
		.accountsStrict({
			initializer: provider.publicKey,
			market,
			nftMint: listingArgs.nftMint,
			order,
			verification,
			orderNftTa,
			initializerNftTa,
			nftProgram: nftProgram ?? PublicKey.default,
			nftTokenProgram,
			sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
			associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			systemProgram: SystemProgram.programId,
			program: marketplaceProgramId,
			eventAuthority,
		})
		.remainingAccounts(remainingAccounts)
		.instruction();

	return ix;
};

// Bid on NFT
export type BidArgs = {
	marketIdentifier: string;
	nftMint: string | undefined; // Can be default key or a specific NFT or other identifier
	paymentMint: string;
	price: number;
	size: number;
};

export const getBid = async (provider: Provider, biddingArgs: BidArgs) => {
	const marketProgram = getMarketplaceProgram(provider);
	const market = getMarketPda(biddingArgs.marketIdentifier);
	const initializer = provider.publicKey?.toString();
	if (!initializer) {
		return undefined;
	}

	const paymentTokenProgram = await getTokenProgramFromMint(provider, biddingArgs.paymentMint.toString());
	if (!paymentTokenProgram) {
		return undefined;
	}

	const nonceKp = Keypair.generate();
	const nonce = nonceKp.publicKey;

	const order = getOrderAccount(nonce.toString(), market.toString(), initializer);
	const initializerPaymentTa = getAtaAddress(biddingArgs.paymentMint, initializer, paymentTokenProgram.toString());
	const orderPaymentTa = getAtaAddress(biddingArgs.paymentMint, order.toString(), paymentTokenProgram.toString());

	const eventAuthority = getEventAuthority();

	const ix = await marketProgram.methods
		.bid({
			nonce,
			price: new BN(biddingArgs.price),
			size: new BN(biddingArgs.size),
		})
		.accountsStrict({
			initializer: provider.publicKey,
			market,
			nftMint: biddingArgs.nftMint ?? PublicKey.default,
			order,
			orderPaymentTa,
			initializerPaymentTa,
			paymentTokenProgram,
			associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			systemProgram: SystemProgram.programId,
			program: marketplaceProgramId,
			eventAuthority,
			paymentMint: biddingArgs.paymentMint,
		})
		.instruction();

	return ix;
};

export const fillOrder = async (provider: Provider, orderAddress: string, nftMint: string, extraAccountParams: WnsAccountParams | undefined) => {
	const marketProgram = getMarketplaceProgram(provider);
	const initializer = provider.publicKey?.toString();
	if (!initializer) {
		return undefined;
	}

	const order = await fetchOrderByAddress(provider, orderAddress);
	if (!order) {
		return undefined;
	}

	const nftTokenProgram = await getTokenProgramFromMint(provider, nftMint);
	const paymentTokenProgram = await getTokenProgramFromMint(provider, order.paymentMint.toString());
	if (!paymentTokenProgram || !nftTokenProgram) {
		return undefined;
	}

	const nftProgram = await getNftProgramFromMint(provider, nftMint);

	const isBuy = order.side === 0;

	const nftRecipient = isBuy ? order.owner.toString() : initializer;
	const nftFunder = isBuy ? initializer : orderAddress;
	const paymentFunder = isBuy ? orderAddress : initializer.toString();
	const paymentRecipient = isBuy ? initializer : order.owner.toString();

	const buyerPaymentTa = getAtaAddress(order.paymentMint.toString(), paymentFunder, paymentTokenProgram.toString());
	const sellerPaymentTa = getAtaAddress(order.paymentMint.toString(), paymentRecipient, paymentTokenProgram.toString());
	const buyerNftTa = getAtaAddress(nftMint, nftFunder, paymentTokenProgram.toString());
	const sellerNftTa = getAtaAddress(nftMint, nftRecipient, paymentTokenProgram.toString());

	const verification = getVerificationPda(order.market.toString(), nftMint);
	const eventAuthority = getEventAuthority();

	const remainingAccounts: AccountMeta[] = await getRemainingAccountsForMint(provider, nftMint, extraAccountParams);

	const ix = await marketProgram.methods
		.fillOrder()
		.accountsStrict({
			taker: provider.publicKey,
			maker: order.owner,
			market: order.market,
			order: orderAddress,
			paymentFunder,
			paymentRecipient,
			nftFunder,
			nftRecipient,
			buyerNftTa,
			buyerPaymentTa,
			sellerNftTa,
			sellerPaymentTa,
			nftTokenProgram,
			paymentTokenProgram,
			nftProgram: nftProgram ?? PublicKey.default,
			associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			systemProgram: SystemProgram.programId,
			program: marketplaceProgramId,
			eventAuthority,
			paymentMint: order.paymentMint,
			nftMint,
			verification,
			sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
		})
		.remainingAccounts(remainingAccounts)
		.instruction();

	return ix;
};

// Cancel Listing
export const cancelListing = async (provider: Provider, orderAddress: PublicKey) => {
	const marketProgram = getMarketplaceProgram(provider);
	const initializer = provider.publicKey?.toString();
	if (!initializer) {
		return undefined;
	}

	const order = await fetchOrderByAddress(provider, orderAddress.toString());
	if (!order) {
		return undefined;
	}

	const {nftMint} = order;

	const nftTokenProgram = await getTokenProgramFromMint(provider, nftMint.toString());
	if (!nftTokenProgram) {
		return undefined;
	}

	const initializerNftTa = getAtaAddress(nftMint.toString(), initializer, nftTokenProgram.toString());
	const orderNftTa = getAtaAddress(nftMint.toString(), orderAddress.toString(), nftTokenProgram.toString());

	const nftProgram = await getNftProgramFromMint(provider, nftMint.toString());

	const eventAuthority = getEventAuthority();

	const ix = await marketProgram.methods
		.cancelListing()
		.accountsStrict({
			initializer: provider.publicKey,
			market: order.market,
			nftMint,
			order: orderAddress,
			orderNftTa,
			initializerNftTa,
			nftProgram: nftProgram ?? PublicKey.default,
			nftTokenProgram,
			sysvarInstructions: SYSVAR_INSTRUCTIONS_PUBKEY,
			associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			systemProgram: SystemProgram.programId,
			program: marketplaceProgramId,
			eventAuthority,
		})
		.instruction();

	return ix;
};

// Cancel Bid
export const cancelBid = async (provider: Provider, orderAddress: PublicKey) => {
	const marketProgram = getMarketplaceProgram(provider);
	const initializer = provider.publicKey?.toString();
	if (!initializer) {
		return undefined;
	}

	const order = await fetchOrderByAddress(provider, orderAddress.toString());
	if (!order) {
		return undefined;
	}

	const paymentMint = order.paymentMint.toString();
	const paymentTokenProgram = await getTokenProgramFromMint(provider, paymentMint);
	if (!paymentTokenProgram) {
		return undefined;
	}

	const initializerPaymentTa = getAtaAddress(paymentMint, initializer, paymentTokenProgram.toString());
	const orderPaymentTa = getAtaAddress(paymentMint, orderAddress.toString(), paymentTokenProgram.toString());

	const eventAuthority = getEventAuthority();

	const ix = await marketProgram.methods
		.cancelBid()
		.accountsStrict({
			initializer: provider.publicKey,
			market: order.market,
			paymentMint,
			order: orderAddress,
			orderPaymentTa,
			initializerPaymentTa,
			paymentTokenProgram,
			associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
			systemProgram: SystemProgram.programId,
			program: marketplaceProgramId,
			eventAuthority,
		})
		.instruction();

	return ix;
};
