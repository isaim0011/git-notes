FROM golang:1.27-alpine AS builder

WORKDIR /app
COPY go.mod ./
# COPY go.sum ./
RUN go mod download

COPY . .
RUN go build -o /github-bridge ./main.go

FROM alpine:3.20

RUN apk add --no-cache git
COPY --from=builder /github-bridge /github-bridge

ENTRYPOINT ["/github-bridge", "sync"]
