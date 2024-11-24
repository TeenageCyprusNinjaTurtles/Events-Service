run-psql:
	docker ps -a --filter name=some-postgres | grep -q . && docker stop some-postgres || echo "Container 'some-postgres' does not exist."
	docker ps -a -q -f name=some-postgres | grep -q . && docker rm some-postgres || echo "Container 'some-postgres' does not exist."
	docker run --name some-postgres -e POSTGRES_PASSWORD=cHt0UFBbszX0YK7 -p 5432:5432  -d postgres