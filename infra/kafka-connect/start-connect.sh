#!/bin/bash
set -e

MAX_ATTEMPTS=10
RETRY_INTERVAL=5

echo "Starting Kafka Connect..."
/etc/confluent/docker/run &

echo "Waiting for Kafka Connect to become available..."
attempt=1
while [ $attempt -le $MAX_ATTEMPTS ]; do
  status=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8083/connectors || echo "000")
  if [ "$status" == "200" ]; then
    echo "Kafka Connect is ready!"
    break
  fi
  echo "Attempt $attempt/$MAX_ATTEMPTS: Kafka Connect not ready yet (status: $status), waiting $RETRY_INTERVAL seconds..."
  sleep $RETRY_INTERVAL
  attempt=$((attempt + 1))
done

if [ $attempt -gt $MAX_ATTEMPTS ]; then
  echo "Failed to connect to Kafka Connect after $MAX_ATTEMPTS attempts. Exiting."
  exit 1
fi

sleep 3

echo "Creating connectors..."
for config_file in /etc/kafka-connect/connector-configs/*.json; do
  connector_name=$(basename "$config_file" .json)
  echo "Creating connector from $config_file..."

  envsubst '${POSTGRES_PASSWORD},${POSTGRES_USER},${POSTGRES_DB}, ${RATINGS_DB_NAME}, ${CLICKHOUSE_USERNAME}, ${CLICKHOUSE_PASSWORD}, ${CLICKHOUSE_DB}, ${RATINGS_DB}' < "$config_file" > /tmp/connector.json
  
  response=$(curl -s -X POST -H "Content-Type: application/json" \
    --data @/tmp/connector.json http://localhost:8083/connectors)
  
  status=$?
  if [ $status -eq 0 ]; then
    echo "Successfully created connector from $config_file"
  else
    echo "Failed to create connector from $config_file (curl exit code: $status)"
    exit 1
  fi
done

rm /tmp/connector.json

echo "All connectors created successfully!"

wait