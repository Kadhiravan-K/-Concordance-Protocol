# Revocation

The issuer of a TOE creates a signed `REVOKE_ECHO` with an increasing sequence
number. The receiver verifies the signer against the TOE, rejects duplicates or
older sequences, records the TOE as revoked, and recomposes every still-active
decision using it. The in-memory MVP demonstrates this behavior; delivery logs,
durable queues, and remote fan-out are federated-pilot work.
