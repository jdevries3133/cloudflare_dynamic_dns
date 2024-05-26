SHELL := /bin/sh

# The registry is presumed to be docker.io, which is the implicit default
DOCKER_ACCOUNT=jdevries3133
CONTAINER_NAME=cloudflare-dynamic-dns
ifdef GITHUB_SHA
	TAG=$(GITHUB_SHA)
else
	TAG=$(shell git rev-parse HEAD)
endif
CONTAINER_QUALNAME=$(DOCKER_ACCOUNT)/$(CONTAINER_NAME)
CONTAINER_EXACT_REF=$(DOCKER_ACCOUNT)/$(CONTAINER_NAME):$(TAG)

build-container:
	rustup target add x86_64-unknown-linux-musl
	cargo build --release --target x86_64-unknown-linux-musl
	docker buildx build --load --platform linux/amd64 -t $(CONTAINER_EXACT_REF) .

# Run the above container locally, such that it can talk to the local
# PostgreSQL database launched by `make _start-db`. We expect here that the
# local database is already running and the container has already been built.
debug-container:
	$(ENV) docker run \
		-e RUST_BACKTRACE=1 \
		-e DATABASE_URL="$$DATABASE_URL" \
		-e SESSION_SECRET="$$SESSION_SECRET" \
		-p 8000:8000 \
		$(CONTAINER_EXACT_REF)

push-container: build-container
	docker push $(CONTAINER_EXACT_REF)

check:
	cargo clippy -- -D warnings
	cargo fmt --check
	terraform fmt --check
	cargo test

deploy:
ifdef CI
	terraform init
endif
	terraform apply -auto-approve
