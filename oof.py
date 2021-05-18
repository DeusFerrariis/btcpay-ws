import hmac, hashlib, base64

body = '{"invoiceId": "some-id", "type": "InvoicePaymentRecieved"}'

print(hmac.new(b"some-key", bytes(body, 'utf-8') , hashlib.sha256).hexdigest())
