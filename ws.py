import asyncio
import websockets
import time

async def howmdy(uri):
    async with websockets.connect(uri) as ws:
        while True:
            try:
                print((await ws.recv()))
            except:
                print("Websocket closed")
                break
            time.sleep(1)

asyncio.get_event_loop().run_until_complete(
    howmdy('ws://localhost:5000/ws?invoice_id=some-id'))
