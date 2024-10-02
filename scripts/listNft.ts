import { Keypair, MessageV0, PublicKey, Transaction, VersionedTransaction } from "@solana/web3.js";
import { BidArgs, getCancelBid, fetchOrdersByMarket, fetchOrdersByMint, fetchOrdersByUser, fillOrder, getBid, getCancelListing, getComputeBudgetInstructions, getInitializeMarket, getListNft, getMarketPda, getOrderAccount, getProvider, getVerifyMint, ListArgs } from "../clients/rarible-svm-ts/src";

const demoMarket = {
    marketIdentifier: new PublicKey("93zMBLNjEVpnT1jk9CHMscM7wbT7rE2ydvCxd3XGM1Vy"),
    wnsGroup: "EXJB8YFiBpFeBbjq9PZhCsav2RfcZVeuBvT9t3jBuY5f",
    paymentMint: "GRhhJGyjkHYcVmvKTb8okbPZD6HMYZrAcWpyN4R3bK5n",
};

const mintsToVerify = ["4swpX19vkSyd2TLM2W2FcsHorkvh25aEg4b35H4Vf2Wv", "71xkPoBLf5DRmCkPJo9czuNL3docUuweeDARABcNsaVy", "CchC7gtAUL1bDbuomRkjupySAZcMAygRb983r3qnkXFv"];

const listingData = [{
        nftMint: "4swpX19vkSyd2TLM2W2FcsHorkvh25aEg4b35H4Vf2Wv",
        price: 150000,
    },
    {
        nftMint: "71xkPoBLf5DRmCkPJo9czuNL3docUuweeDARABcNsaVy",
        price: 240000,
    }
];

const biddingData = [
    {
    price: 75000,
    size: 2,
    nftMint: undefined,
},
{
    price: 150000,
    size: 2,
    nftMint: undefined,
}
];

const nftToSell = "4swpX19vkSyd2TLM2W2FcsHorkvh25aEg4b35H4Vf2Wv";

async function setupMarket() {
    const provider = getProvider();
    const { marketIdentifier } = demoMarket;
    const createMarketIx = await getInitializeMarket(provider, marketIdentifier.toString());
    const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
    const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [createMarketIx], recentBlockhash, })
    const tx = new VersionedTransaction(message);
    const signedTx = await provider.wallet.signTransaction(tx);

    try{
        const txSig = await provider.connection.sendTransaction(signedTx);
        console.log("Create Market --", txSig);
    } catch (e) {
        console.log(e);
    }
}

async function verifyMints() {
    const provider = getProvider();
    const { marketIdentifier } = demoMarket;

    const marketAddress = getMarketPda(marketIdentifier.toString());

    const addMintIxs = await Promise.all(mintsToVerify.map((m) => getVerifyMint(provider, m, marketAddress.toString())));
    for (let i = 0; i < addMintIxs.length; i = i + 5) {
        const mintIxs = addMintIxs.slice(i, Math.min(addMintIxs.length, i + 5));
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [...mintIxs], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);
    
        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("Verify --", txSig);
        } catch (e) {
            console.log(e);
        }
    }
}

async function listNfts() {
    const provider = getProvider();
    const { marketIdentifier, paymentMint, wnsGroup } = demoMarket;

    const wnsParams = {
        groupMint: wnsGroup,
        paymentMint
    };
    const listings: ListArgs[] = listingData.map((m) => { return {
        marketIdentifier: marketIdentifier.toString(),
        nftMint: m.nftMint,
        paymentMint,
        price: m.price,
        extraAccountParams: wnsParams,
    }})
    
    const listNftIxs = await Promise.all(listings.map((l) => getListNft(provider, l)));

    for (let i = 0; i < listNftIxs.length; i++) {
        const computeIxs = getComputeBudgetInstructions({ computeUnits: 300_000 });
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [...computeIxs, listNftIxs[i]], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);

        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("List --", txSig);
        } catch (e) {
            console.log(e);
        }
    }
}

async function bid() {
    const provider = getProvider();
    const { marketIdentifier, paymentMint } = demoMarket;

    const bids: BidArgs[] = biddingData.map(b => {
        return {
            marketIdentifier: marketIdentifier.toString(),
            paymentMint,
            price: b.price,
            size: b.size,
            nftMint: b.nftMint,
        }
    });

    const bidNftIxs = await Promise.all(bids.map((b) => getBid(provider, b)));

    for (let i = 0; i < bidNftIxs.length; i++) {
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [bidNftIxs[i]], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);

        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("Bid --", txSig);
        } catch (e) {
            console.log(e);
        }
    }
}

async function sellNft() {
    const provider = getProvider();
    const { marketIdentifier, paymentMint, wnsGroup } = demoMarket;

    const marketAddress = getMarketPda(marketIdentifier.toString());

    const wnsParams = {
        groupMint: wnsGroup,
        paymentMint
    };

    const activeOrders = (await fetchOrdersByMarket(provider, marketAddress.toString())).filter((o) => o.account.state == 0).filter(o => o !== undefined);
    const topBid = activeOrders.filter(o => o.account.side == 0).sort((a, b) => b.account.price.toNumber() - a.account.price.toNumber()).pop();

    if (topBid !== undefined) {
        const buyNftIx = await fillOrder(provider, topBid.publicKey.toString(), nftToSell, wnsParams);
        
        const computeIxs = getComputeBudgetInstructions({ computeUnits: 500_000 });
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [...computeIxs, buyNftIx], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);
    
        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("Buy --", txSig);
        } catch (e) {
            console.log(e);
        }
    }

}

async function buyNft() {
    const provider = getProvider();
    const { marketIdentifier } = demoMarket;

    const marketAddress = getMarketPda(marketIdentifier.toString());

    const {paymentMint, wnsGroup} = demoMarket;
    const wnsParams = {
        groupMint: wnsGroup,
        paymentMint
    };
   
    const activeOrders = (await fetchOrdersByMarket(provider, marketAddress.toString())).filter((o) => o.account.state == 0).filter(o => o !== undefined);
    const floorNft = activeOrders.filter(o => o.account.side == 1).sort((a, b) => a.account.price.toNumber() - b.account.price.toNumber()).pop();

    if (floorNft !== undefined) {
        const buyNftIx = await fillOrder(provider, floorNft.publicKey.toString(), floorNft.account.nftMint.toString(), wnsParams);
        
        const computeIxs = getComputeBudgetInstructions({ computeUnits: 500_000 });
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [...computeIxs, buyNftIx], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);
    
        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("Buy --", txSig);
        } catch (e) {
            console.log(e);
        }
    }
}


async function cancelListings() {
    const provider = getProvider();
    const { paymentMint, wnsGroup } = demoMarket;

    const wnsParams = {
        groupMint: wnsGroup,
        paymentMint
    };
   
    const bidsForUser = (await fetchOrdersByUser(provider, provider.publicKey.toString())).filter(o => o.account.state == 0).filter(o => o.account.side == 1);

    const cancelListingsIxs = await Promise.all(bidsForUser.map(o => getCancelListing(provider, o.publicKey, wnsParams)));

    for (let i = 0; i < cancelListingsIxs.length; i++) {
        const computeIxs = getComputeBudgetInstructions({ computeUnits: 300_000 });
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [...computeIxs, cancelListingsIxs[i]], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);

        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("Cancel Listing --", txSig);
        } catch (e) {
            console.log(e);
        }
    }
}

async function cancelBids() {
    const provider = getProvider();
   
    const bidsForUser = (await fetchOrdersByUser(provider, provider.publicKey.toString())).filter(o => o.account.state == 0).filter(o => o.account.side == 0);

    const cancelBidIxs = await Promise.all(bidsForUser.map(o => getCancelBid(provider, o.publicKey)));

    for (let i = 0; i < cancelBidIxs.length; i++) {
        const computeIxs = getComputeBudgetInstructions({ computeUnits: 300_000 });
        const recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;
        const message = MessageV0.compile({ payerKey: provider.publicKey, instructions: [...computeIxs, cancelBidIxs[i]], recentBlockhash, })
        const tx = new VersionedTransaction(message);
        const signedTx = await provider.wallet.signTransaction(tx);

        try{
            const txSig = await provider.connection.sendTransaction(signedTx);
            console.log("Cancel Listing --", txSig);
        } catch (e) {
            console.log(e);
        }
    }
}

setupMarket();
// verifyMints();
// listNfts();
// bid();
// sellNfts();
// buyNfts();
// cancelListings();
// cancelBids();