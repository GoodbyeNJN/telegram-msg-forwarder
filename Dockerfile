FROM alpine AS runner

COPY ./server /bin/server

ENV ROCKET_ADDRESS=0.0.0.0

ENTRYPOINT ["/bin/server"]
