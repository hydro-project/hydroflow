import asyncio
import sys

async def wrap(inner):
    try:
        return (await inner(), None)
    except:
        return (None, sys.exc_info())

def run(inner):
    event_loop = asyncio.get_event_loop()
    task = event_loop.create_task(wrap(inner))
    should_cancel = False
    try:
        res = event_loop.run_until_complete(task)
    except:
        should_cancel = True

    if should_cancel:
        task.cancel()
        res = event_loop.run_until_complete(task)

    pending = asyncio.all_tasks(loop=event_loop)
    group = asyncio.gather(*pending)
    event_loop.run_until_complete(group)

    event_loop.close()

    if res[1]:
        raise res[1][1].with_traceback(res[1][2])
    else:
        return res[0]
