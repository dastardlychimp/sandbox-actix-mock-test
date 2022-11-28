db/remake: db/build db/down db/run

db/build:
	docker build -t sandbox-pg .

db/run: 
	docker run -p 5995:5432 -d --name sandbox-pg sandbox-pg 

db/down:
	docker stop sandbox-pg | xargs docker rm

db/psql:
	psql -U postgres -h localhost -p 5995 -d sandbox