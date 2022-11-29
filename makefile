.PHONY: all
all: db/remake build

.PHONY: db/remake
db/remake: db/build db/down db/run

.PHONY: db/build
db/build:
	docker build -t sandbox-pg .

.PHONY: db/run
db/run: 
	docker run -p 5995:5432 -d --name sandbox-pg sandbox-pg 

.PHONY: db/down
db/down:
	docker stop sandbox-pg | xargs docker rm

.PHONY: db/psql
db/psql:
	psql -U postgres -h localhost -p 5995 -d sandbox

.PHONY: build
build:
	cargo build