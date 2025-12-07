#!/bin/bash
# Simulates Rails log output for testing including N+1 queries

echo "=== Request 1: Simple query ==="
echo "Started GET \"/users/123\" for 127.0.0.1 at 2024-01-01 12:00:00"
sleep 0.3
echo "Processing by UsersController#show as HTML"
sleep 0.2
echo "  User Load (0.5ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 123 LIMIT 1"
sleep 0.2
echo "Completed 200 OK in 15.7ms (Views: 8.4ms | ActiveRecord: 2.1ms)"
sleep 1.5

echo "=== Request 2: N+1 Query Example ==="
echo "Started GET \"/posts\" for 127.0.0.1 at 2024-01-01 12:00:05"
sleep 0.3
echo "Processing by PostsController#index as HTML"
sleep 0.2
echo "  Post Load (1.2ms)  SELECT \"posts\".* FROM \"posts\" ORDER BY created_at DESC LIMIT 10"
sleep 0.3
# Simulate N+1: loading author for each post
echo "  User Load (0.4ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 1 LIMIT 1"
sleep 0.2
echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 2 LIMIT 1"
sleep 0.2
echo "  User Load (0.4ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 3 LIMIT 1"
sleep 0.2
echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 4 LIMIT 1"
sleep 0.2
echo "  User Load (0.4ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 5 LIMIT 1"
sleep 0.2
echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 6 LIMIT 1"
sleep 0.2
echo "  User Load (0.4ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 7 LIMIT 1"
sleep 0.2
echo "Completed 200 OK in 85.3ms (Views: 45.2ms | ActiveRecord: 38.1ms)"
sleep 2

echo "=== Request 3: Another request ==="
echo "Started POST \"/articles\" for 127.0.0.1 at 2024-01-01 12:00:10"
sleep 0.3
echo "Processing by ArticlesController#create as HTML"
sleep 0.2
echo "  BEGIN (0.1ms)"
sleep 0.1
echo "  INSERT INTO \"articles\" (\"title\", \"body\", \"created_at\") VALUES ('Test', 'Content', NOW()) (1.2ms)"
sleep 0.2
echo "  COMMIT (0.3ms)"
sleep 0.1
echo "Completed 201 Created in 23.5ms (ActiveRecord: 5.6ms)"
sleep 2

echo "=== Request 4: More N+1 (comments) ==="
echo "Started GET \"/articles/5\" for 127.0.0.1"
sleep 0.3
echo "Processing by ArticlesController#show as HTML"
sleep 0.2
echo "  Article Load (0.6ms)  SELECT \"articles\".* FROM \"articles\" WHERE \"articles\".\"id\" = 5 LIMIT 1"
sleep 0.2
echo "  Comment Load (1.1ms)  SELECT \"comments\".* FROM \"comments\" WHERE \"comments\".\"article_id\" = 5"
sleep 0.3
echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 10 LIMIT 1"
sleep 0.2
echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 11 LIMIT 1"
sleep 0.2
echo "  User Load (0.4ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 12 LIMIT 1"
sleep 0.2
echo "  User Load (0.3ms)  SELECT \"users\".* FROM \"users\" WHERE \"users\".\"id\" = 13 LIMIT 1"
sleep 0.2
echo "Completed 200 OK in 45.8ms (Views: 25.3ms | ActiveRecord: 18.5ms)"
sleep 2

while true; do
  echo "Started GET \"/api/health\" for 127.0.0.1"
  sleep 0.1
  echo "Completed 200 OK in 2.3ms"
  sleep 5
done
