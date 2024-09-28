# dm-reports

Simple DM reporting system for discord. Easily verifiable reports of users scamming and spamming in DMs.

## Setup

I assume you know how to use docker. If you don't, [maybe ask in my discord](https://valk.sh/discord).

### Register commands

```bash
docker run -e DISCORD_TOKEN="<your token>"  ghcr.io/randomairborne/dm-reports-create-cmds:latest -- "<your server name>"
```

### Run

`ghcr.io/randomairborne/dm-reports:latest`
Needs environment `VERIFY_KEY` from the bot dashboard public key, and the `WEBHOOK_URL` that you want your
reports to be sent to. You can point your discord interactions endpoint at `container:8080/api/interactions`.
