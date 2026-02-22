CREATE USER user_api WITH PASSWORD 'secret';
--CREATE DATABASE api_database WITH ENCODING 'UTF8' OWNER user_api;
GRANT ALL ON DATABASE api_database TO user_api;
