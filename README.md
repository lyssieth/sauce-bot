# sauce-bot

A bot which finds the original source of images, often referred to as 'sauce'.

## Usage

The bot uses the prefix `sauce!`.

### Currently implemented

- `sauce!nao <link>` to use the saucenao backend (faster but rate limited, more places searched)
  - Global (across all users) rate limits currently:
    - 30s: 6 searches
    - 24h: 200 searches
- `sauce!iqdb <link>` to use the iqdb backend (slower but no rate limit, less places searched)
- `sauce!issue` to get a direct link to the issues page
- `sauce!help` to provide some help.

## Links

- [Bot Invite Link](https://discord.com/oauth2/authorize?client_id=778822593293058051&scope=bot&permissions=19456)
- [My Patreon](https://patreon.com/lyssieth)
