FROM confluentinc/cp-kafka-connect:7.9.0

USER root
RUN yum install -y gettext && yum clean all

RUN confluent-hub install --no-prompt debezium/debezium-connector-postgresql:2.5.4 && \
    confluent-hub install --no-prompt confluentinc/kafka-connect-elasticsearch:14.1.2 && \
    confluent-hub install --no-prompt confluentinc/kafka-connect-jdbc:10.8.4 && \
    confluent-hub install --no-prompt clickhouse/clickhouse-kafka-connect:v1.3.1