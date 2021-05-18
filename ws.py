import asyncio
import websockets

async def howmdy(uri):
    async with websockets.connect(uri) as ws:
        print((await ws.recv()))

asyncio.get_event_loop().run_until_complete(
    howmdy('ws://localhost:5000/ws?invoice_id=some-id'))
