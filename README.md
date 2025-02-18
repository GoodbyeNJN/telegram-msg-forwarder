# Telegram Message Forwarder

A webhook for forwarding text messages to Telegram.

## Usage

Start the server with the following compose file:

```yaml
services:
  forwarder:
    image: ghcr.io/goodbyenjn/telegram-msg-forwarder
    container_name: forwarder
    restart: unless-stopped
    ports:
      - 8080:8080
    environment:
      - TELEGRAM_BOT_TOKEN=<token_from_botfather>
      - TELEGRAM_CHAT_ID=<chat_id>
      # (Required) The auth token(s) to protect the endpoint, separated by spaces
      - AUTH_TOKEN=<auth_token_1> <auth_token_2> <auth_token_3>
```

Then, send a POST request to the server with a JSON body:

```bash
curl -X POST -H "Authorization: Bearer <auth_token>" -d '{"title": "Forwarded Message", "message": "Hello, world!"}' http://localhost:8080/api/forward
```

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For any questions or suggestions, please open an issue or contact the repository owner.
