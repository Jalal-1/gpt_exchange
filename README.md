# gpt_exchange

GPT Exchange Oracle

## Installation and usage

1. ```sqlx database setup``` to run the migrations
2. ```cargo run --bin httpd``` to start the http daemon
3. ```cargo run --bin jobclient -- --api-key <api-key> new --help``` to see the available commands (new terminal)
4. Navigate to ```127.0.0.1/8000/api/job/key``` to generate an api key (it will be displayed in the terminal)
