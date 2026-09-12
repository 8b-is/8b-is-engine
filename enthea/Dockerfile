# syntax=docker/dockerfile:1

# --- build stage: pure stdlib Go, static, no CGO ---
FROM golang:1.26-alpine AS build
WORKDIR /src
COPY go.mod ./
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -trimpath -ldflags="-s -w" -o /out/enthea .

# --- runtime stage: ultra-pure, hardened scratch ---
# no shell · no package manager · no binaries but one · no attack surface
FROM scratch
COPY --from=build /out/enthea /enthea

# hardened: non-root (the static binary needs no /etc/passwd), no caps
USER 65532:65532
WORKDIR /

# the engine door, as a service: `enthea mcp` over stdin/stdout
ENTRYPOINT ["/enthea"]
CMD ["mcp"]

LABEL org.opencontainers.image.source=https://github.com/8b-is/enthea
LABEL org.opencontainers.image.description="enthea — the deepsiper-enthea engine door. One static binary, no deps, no shell."
LABEL org.opencontainers.image.licenses=MIT
