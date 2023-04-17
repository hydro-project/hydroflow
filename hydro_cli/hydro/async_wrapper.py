import asyncio
import sys
import gc

async def wrap(inner, args):
    try:
        return (await inner(args), None)
    except asyncio.CancelledError:
        exc = sys.exc_info()
        return (None, [exc[0], exc[1].with_traceback(None), None])
    except:
        return (None, sys.exc_info())

def run(inner, args):
    event_loop = asyncio.get_event_loop()
    task = event_loop.create_task(wrap(inner, args))
    should_cancel = False

    try:
        res = event_loop.run_until_complete(task)
    except KeyboardInterrupt:
        should_cancel = True
        # don't re-raise the exception, to give Rust a chance to gracefully shut down
        res = (None, [KeyboardInterrupt, KeyboardInterrupt("Received keyboard interrupt"), None])
    except:
        should_cancel = True
        cancel_reason = sys.exc_info()

        # avoid leaking references to the coroutine
        cancel_reason[1].__traceback__ = None
        res = (None, [cancel_reason[0], cancel_reason[1], None])
        del cancel_reason

    if should_cancel:
        task.cancel()
        try:
            event_loop.run_until_complete(task)
        except asyncio.CancelledError:
            pass

    for task in asyncio.all_tasks(loop=event_loop):
        task.cancel()
        try:
            event_loop.run_until_complete(task)
        except asyncio.CancelledError:
            pass

    event_loop.close()

    del task
    del event_loop

    gc.collect()

    if res[1]:
        raise res[1][1].with_traceback(res[1][2])
    else:
        return res[0]
