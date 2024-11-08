export ANCHOR_WALLET=~/.config/solana/id.json
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com

npx ts-node ./src/list.ts \
  -k ~/.config/solana/id.json \
  -r https://api.devnet.solana.com \
  -m "674s1Sap3KVnr8WGrY5KGQ69oTYjjgr1disKJo6GpTYw" \
  --nftMint 5Y5WFuQg5TdscMgNvxL72qKGbob9nTiqG9fQyssWD2VJ \
  --paymentMint So11111111111111111111111111111111111111112 \
  --size 1 \
  --price 1000000000


# success tx on sol dev net 5imDxnAiF2Af6ZX1u9UXp3ecqD7CSTLuY4xPaBz4XoSNY8sFJATfYo3E4z2JeTSVyD4pWgXNE8kUvKGXoi84vPfx