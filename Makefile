build:
	cargo build --release
docker-image:
	docker build -t rust-build $(shell pwd)
ci: docker-image
	# docker run -v $(shell pwd):/hayoo-cli -ti rust-build bash
	docker run -v $(shell pwd):/code -t rust-build bash -c "cd /code && make build"
