.PHONY: mock

mock: mock_data.sql
	docker exec -i accounts_rs-db-1 psql -U accounts_rs accounts_rs < mock_data.sql