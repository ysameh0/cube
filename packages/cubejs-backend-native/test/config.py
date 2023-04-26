from cube.conf import settings

settings.schema_path = "models"
settings.pg_sql_port = 5555
settings.telemetry = False
settings.query_rewrite = lambda query, ctx: print('query=', query, 'ctx=', ctx)
settings.check_auth = lambda req, authorization: print('req=', req, 'authorization=', authorization)