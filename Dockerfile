FROM postgres

ENV POSTGRES_HOST_AUTH_METHOD=trust
ENV POSTGRES_DB=sandbox
ENV POSTGRES_USER=postgres

COPY ./migrations /docker-entrypoint-initdb.d