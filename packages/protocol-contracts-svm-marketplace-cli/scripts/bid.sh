# GoBYFSqWdqMNMP9L59EZ2g8wQvQ8o4ZzHzpjLgEmnu68 -- order id

#!/bin/bash
# solana-keygen new --outfile ~/.config/solana/id-buyer.json
# solana transfer AgyQbLU5fpXnAJHJBg4BauhQK26dT5UPhMgNgfhjSBQN 5 --allow-unfunded-recipient
# Replace these variables with your actual values
export ANCHOR_WALLET=~/.config/solana/id-buyer.json
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com

ANCHOR_WALLET=~/.config/solana/id-buyer.json
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com

KEYPAIR_PATH=/Users/vfadeev/.config/solana/id-buyer.json
RPC_ENDPOINT="https://api.devnet.solana.com"
MARKET_IDENTIFIER="674s1Sap3KVnr8WGrY5KGQ69oTYjjgr1disKJo6GpTYw"
NFT_MINT="5Y5WFuQg5TdscMgNvxL72qKGbob9nTiqG9fQyssWD2VJ" # Replace with the actual NFT mint address
PAYMENT_MINT="So11111111111111111111111111111111111111112" # SOL mint address
PRICE="100000000" # Price in lamports (1 SOL = 1,000,000,000 lamports)
SIZE="1"

# Call the bidNft.ts script
npx ts-node ./src/bid.ts \
  -k "$KEYPAIR_PATH" \
  -r "$RPC_ENDPOINT" \
  -m "$MARKET_IDENTIFIER" \
  --nftMint "$NFT_MINT" \
  --paymentMint "So11111111111111111111111111111111111111112" \
  --price "$PRICE" \
  --size "$SIZE"