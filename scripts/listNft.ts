import { Keypair, Transaction } from "@solana/web3.js";
import { BidArgs, getBid, getInitializeMarket, getListNft, getMarketPda, getProvider, getVerifyMint, ListArgs } from "../clients/rarible-svm-ts/src";

async function setupMarket() {
    const provider = getProvider();
    const marketIdentifier = Keypair.generate().publicKey;

    const tx = new Transaction();
    const createMarketIx = await getInitializeMarket(provider, marketIdentifier.toString());
    tx.add(createMarketIx);

    const marketAddress = getMarketPda(marketIdentifier.toString());

    const paymentMint = "";
    const groupMint = ""; // WNS Test

    const nft1 = "";
    const nft2 = "";

    const mintsToAdd = [
        nft1,
        nft2
    ];

    const addMintIxs = await Promise.all(mintsToAdd.map((m) => getVerifyMint(provider, m, marketAddress.toString())));
    tx.add(...addMintIxs);

    const wnsParams = {
        groupMint,
        paymentMint
    };
    const listings: ListArgs[] = [
        {
            marketIdentifier: marketIdentifier.toString(),
            nftMint: nft1.toString(),
            paymentMint,
            price: 150000,
            extraAccountParams: wnsParams,
        },
        {
            marketIdentifier: marketIdentifier.toString(),
            nftMint: nft2.toString(),
            paymentMint,
            price: 250000,
            extraAccountParams: wnsParams,
        }
    ];
    
    const listNftIxs = await Promise.all(listings.map((l) => getListNft(provider, l)));

    for (let i = 0; i < listNftIxs.length; i++) {
        const tx = new Transaction();
        tx.add(listNftIxs[i]);

        const txSig = await provider.connection.sendTransaction(tx, []);
        console.log({txSig});
    }
    const bids: BidArgs[] = [
        {
            marketIdentifier: marketIdentifier.toString(),
            paymentMint,
            price: 75000,
            size: 4,
            nftMint: undefined,
        }, 
        {
            marketIdentifier: marketIdentifier.toString(),
            paymentMint,
            price: 100000,
            size: 2,
            nftMint: undefined,
        }
    ];


    const bidNftIxs = await Promise.all(bids.map((b) => getBid(provider, b)));

    for (let i = 0; i < bidNftIxs.length; i++) {
        const tx = new Transaction();
        tx.add(bidNftIxs[i]);

        const txSig = await provider.connection.sendTransaction(tx, []);
        console.log({txSig});
    }
}