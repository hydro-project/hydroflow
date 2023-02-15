from typing import Optional
import hydro_cli_rust # type: ignore

class Deployment(object):
    def __init__(self) -> None:
        self.underlying = hydro_cli_rust.create_Deployment()

    def Localhost(self) -> "Localhost":
        return Localhost(self)

    def HydroflowCrate(self, src: str, on: "Host", example: Optional[str] = None) -> "HydroflowCrate":
        return HydroflowCrate(src, on, example, self)

    def deploy(self):
        return self.underlying.deploy()

class Host(object):
    def __init__(self, underlying) -> None:
        self.underlying = underlying

    def provision(self):
        return self.underlying.provision()

class Localhost(Host):
    def __init__(self, deployment: Deployment):
        super().__init__(hydro_cli_rust.create_LocalhostHost(deployment.underlying))

class Service(object):
    def __init__(self, underlying) -> None:
        self.underlying = underlying

    def deploy(self):
        return self.underlying.deploy()

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

class HydroflowCrate(Service):
    def __init__(self, src: str, on: Host, example: Optional[str], deployment: Deployment) -> None:
        super().__init__(hydro_cli_rust.create_HydroflowCrate(src, on.underlying, example, deployment.underlying))

    @property
    def ports(self) -> HydroflowCratePorts:
        return HydroflowCratePorts(self.underlying)
