from typing import Union, Callable

class RequestContext:
    pass

class Configuration:
    def __init__(self):
        self.schema_path = None
        self.base_path = None
        self.db_type = None
        self.context_to_app_id = None
        self.context_to_orchestrator_id = None
        self.compiler_cache_size = None
        self.telemetry = None
        self.pg_sql_port = None

    def set_schema_path(self, schema_path: str):
        self.schema_path = schema_path

    def set_base_path(self, base_path: str):
        self.base_path = base_path

    def set_db_type(self, db_type: Union[str, Callable[[RequestContext], str]]):
        self.db_type = db_type

    def set_context_to_app_id(self, context_to_app_id: Union[str, Callable[[RequestContext], str]]):
        self.context_to_app_id = context_to_app_id

    def set_context_to_orchestrator_id(self, context_to_orchestrator_id: Union[str, Callable[[RequestContext], str]]):
        self.context_to_orchestrator_id = context_to_orchestrator_id

    def set_compiler_cache_size(self, compiler_cache_size: bool):
        self.compiler_cache_size = compiler_cache_size

    def set_telemetry(self, telemetry: bool):
        self.telemetry = telemetry

    def set_pg_sql_port(self, pg_sql_port: int):
        self.pg_sql_port = pg_sql_port

settings = Configuration()
