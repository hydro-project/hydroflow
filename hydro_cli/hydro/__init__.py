import hydro_cli_rust # type: ignore

class Host(object):
    def __init__(self, underlying) -> None:
        self.underlying = underlying

    def provision(self):
        return self.underlying.provision()

class Localhost(Host):
    def __init__(self):
        super().__init__(hydro_cli_rust.create_LocalhostHost())

class HydroflowCrate(object):
    def __init__(self, src: str, on: Host) -> None:
        pass
