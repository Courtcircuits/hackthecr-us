Just to generate a key for testing :

## Generate a private and public DER key using OpenSSL

### 1. Generate a private key (PEM format)

```bash
openssl genpkey -algorithm ed25519 -out private_key.pem
```

### 2. Convert the private key to DER format

```bash
openssl pkcs8 -topk8 -inform PEM -outform DER -in private_key.pem -out private_key.der -nocrypt
```

### 3. Extract the public key in DER format

```bash
openssl pkey -in private_key.pem -pubout -outform DER -out public_key.der
```

### Summary

| File              | Description                          |
|-------------------|--------------------------------------|
| `private_key.pem` | Ed25519 private key (PEM, intermediate) |
| `private_key.der` | Ed25519 private key (DER, PKCS#8)    |
| `public_key.der`  | Ed25519 public key (DER)             |
