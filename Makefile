.PHONY: mock, migrate, clean, reset

DB_CONTAINER_NAME=accounts_rs-db-1

mock: mock_data.sql
	docker exec -i $(DB_CONTAINER_NAME) psql -U accounts_rs accounts_rs < mock_data.sql

migrate:
	cd backend && cargo sqlx migrate run && cd ..

clean:
	echo 'DROP SCHEMA public CASCADE; CREATE SCHEMA public;' | docker exec -i $(DB_CONTAINER_NAME) psql -U accounts_rs accounts_rs

reset:
	make clean
	make migrate
	make mock
