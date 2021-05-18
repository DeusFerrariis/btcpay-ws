import hmac, hashlib, base64

body = '{"invoiceId": "some-id", "type": "InvoicePayed"}'

print(hmac.new(b"some-key", bytes(body, 'utf-8') , hashlib.sha256).hexdigest())
