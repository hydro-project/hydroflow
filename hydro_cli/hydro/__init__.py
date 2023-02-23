from typing import List, Optional
import hydro_cli_rust # type: ignore

class Deployment(object):
    def __init__(self) -> None:
        self.underlying = hydro_cli_rust.PyDeployment()

    def Localhost(self) -> "Localhost":
        return Localhost(self)

    def HydroflowCrate(self, src: str, on: "Host", example: Optional[str] = None, features: Optional[List[str]] = None) -> "HydroflowCrate":
        return HydroflowCrate(self, src, on, example, features)

    def deploy(self):
        return self.underlying.deploy()

    def start(self):
        return self.underlying.start()

class Host(object):
    def __init__(self, underlying) -> None:
        self.underlying = underlying

class Localhost(Host):
    def __init__(self, deployment: Deployment):
        super().__init__(hydro_cli_rust.PyLocalhostHost(deployment.underlying))

class Service(object):
    def __init__(self, underlying) -> None:
        self.underlying = underlying

class HydroflowPort(object):
    def __init__(self, underlying, name) -> None:
        self.underlying = underlying
        self.name = name

    def send_to(self, other: "HydroflowPort"):
        hydro_cli_rust.create_connection(
            self.underlying,
            self.name,
            other.underlying,
            other.name
        )

class HydroflowCratePorts(object):
    def __init__(self, underlying) -> None:
        self.__underlying = underlying

    def __getattribute__(self, __name: str) -> HydroflowPort:
        if __name == "_HydroflowCratePorts__underlying":
            return object.__getattribute__(self, __name)
        return HydroflowPort(self.__underlying, __name)

async def pyreceiver_to_async_generator(pyreceiver):
    while True:
        res = await pyreceiver.next()
        if res is None:
            break
        else:
            yield res

class HydroflowCrate(Service):
    def __init__(self, deployment: Deployment, src: str, on: Host, example: Optional[str], features: Optional[List[str]]) -> None:
        super().__init__(hydro_cli_rust.PyHydroflowCrate(deployment.underlying, src, on.underlying, example, features))

    @property
    def ports(self) -> HydroflowCratePorts:
        return HydroflowCratePorts(self.underlying)

    async def stdout(self):
        return pyreceiver_to_async_generator(await self.underlying.stdout())

    async def stderr(self):
        return pyreceiver_to_async_generator(await self.underlying.stderr())

    def exit_code(self):
        return self.underlying.exit_code();
